package com.twitter.visibility.under_the_hood

import com.twitter.common_internal.analytics.test_user_filter.TestUserFilter
import com.twitter.gizmoduck.thriftscala.Takedowns
import com.twitter.gizmoduck.thriftscala.UserModification
import com.twitter.guano.thriftscala.PctdAction
import com.twitter.guano.thriftscala.PctdActionType
import com.twitter.scalding.Args
import com.twitter.scalding.DateOps
import com.twitter.scalding.DateParser
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.Execution
import com.twitter.scalding.RichDate
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.TypedTsv
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.dalv2.remote_access.AllowCrossDC
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.scalding_internal.job.analytics_batch._
import com.twitter.twadoop.user.gen.thriftscala.CombinedUser
import com.twitter.usersource.snapshot.combined.UsersourceScalaDataset
import com.twitter.visibility.under_the_hood.TakedownLabels.ReasonType
import com.twitter.visibility.under_the_hood.thriftscala.UthDailyAccountLabel
import java.nio.charset.StandardCharsets
import java.time.Instant
import java.time.ZoneOffset
import java.util.Locale
import java.util.TimeZone
import org.apache.thrift.protocol.TJSONProtocol
import org.apache.thrift.transport.TMemoryInputTransport
import scala.util.Try
import twadoop_config.configuration.log_categories.group.gizmoduck.GizmoduckUserModificationsScalaDataset
import twadoop_config.configuration.log_categories.group.useng.AuditServicePctdActionsScalaDataset

class UthPctdAccountTakedownEventsApp {
  import UthPctdAccountTakedownEventsApp._

  implicit val tz: TimeZone = DateOps.UTC

  def runOnDateRange(
    dateRange: DateRange,
    config: UthPctdAccountTakedownEventsConfig
  ): Execution[Unit] = {
    val rangeStartMs = dateRange.start.timestamp
    val pivotMs = dateRange.end.timestamp + 1L
    require(
      rangeStartMs % DayMs == 0 && pivotMs % DayMs == 0,
      s"range must cover whole UTC days; got [$rangeStartMs, $pivotMs)"
    )
    val dayStarts: Seq[Long] =
      Iterator.iterate(rangeStartMs)(_ + DayMs).takeWhile(_ < pivotMs).toSeq

    val pctdActions = DAL
      .read(
        AuditServicePctdActionsScalaDataset,
        DateRange(dateRange.start, dateRange.end + Days(1)))
      .toTypedPipe
    val usersourceUsers = DAL
      .readMostRecentSnapshot(
        UsersourceScalaDataset,
        DateRange(RichDate(pivotMs - DayMs), RichDate(pivotMs))
      )
      .withRemoteReadPolicy(AllowCrossDC)
      .toTypedPipe

    val gizmoduckMods = DAL
      .read(
        GizmoduckUserModificationsScalaDataset,
        DateRange(dateRange.start, dateRange.end + Days(1)))
      .toTypedPipe

    val diffs = takedownDiffs(gizmoduckMods, config.testUserIds)
    val resolutions = resolveKeys(
      deltas(pctdActions, config.testUserIds, rangeStartMs, pivotMs),
      anchor(usersourceUsers, config.testUserIds),
      armEvents(diffs),
      config.reducers
    )
    val rows = accountLabelRows(withheldAtDayStarts(resolutions, dayStarts), config.reducers)

    val debug: Execution[Unit] = config.debugPath match {
      case Some(path) =>
        Execution
          .zip(
            resolutions
              .map(keyDebugRow(_, dayStarts))
              .writeExecution(TypedTsv(s"$path/key_resolutions")),
            diffs.map(diffDebugRow).writeExecution(TypedTsv(s"$path/takedown_diffs"))
          )
          .unit
      case None => Execution.unit
    }

    rows.forceToDiskExecution.zip(debug).flatMap {
      case (cached, _) =>
        Execution
          .sequence(dayStarts.map { dayStartMs =>
            val day = yyyymmddOf(dayStartMs)
            implicit val partitionRange: DateRange =
              DateRange(RichDate(dayStartMs), RichDate(dayStartMs + DayMs - 1L))
            cached
              .filter(_.dayYyyymmdd.contains(day))
              .shard(config.writeShards)
              .writeDALExecution(
                UthDailyAccountTakedownLabelsScalaDataset,
                D.Daily,
                D.Suffix(s"${config.outputPath}/daily_account_takedown_labels"),
                D.Parquet
              )
          }).unit
    }
  }
}

object UthPctdAccountTakedownEventsApp {
  private[under_the_hood] val DayMs: Long = 86400000L

  type Delta = (Long, Boolean, String)

  private[under_the_hood] def yyyymmddOf(ms: Long): Int = {
    val d = Instant.ofEpochMilli(ms).atZone(ZoneOffset.UTC).toLocalDate
    d.getYear * 10000 + d.getMonthValue * 100 + d.getDayOfMonth
  }

  private def inScope(testUserIds: Set[Long], userId: Long): Boolean =
    !TestUserFilter.isTestUserId(userId) && (testUserIds.isEmpty || testUserIds(userId))

  private val ReasonPriority: Map[ReasonType, Int] = Map(
    ReasonType.LegalRequest -> 0,
    ReasonType.BystanderReport -> 1,
    ReasonType.Dmca -> 2,
    ReasonType.UnspecifiedReason -> 3
  )

  private def higherPriority(a: ReasonType, b: ReasonType): ReasonType =
    if (ReasonPriority.getOrElse(a, 9) <= ReasonPriority.getOrElse(b, 9)) a else b

  private[under_the_hood] def anchor(
    usersourceUsers: TypedPipe[CombinedUser],
    testUserIds: Set[Long]
  ): TypedPipe[((Long, String), ReasonType)] =
    usersourceUsers
      .flatMap { cu =>
        for {
          u <- cu.user.toSeq
          if inScope(testUserIds, u.id)
          t <- u.takedowns.toSeq
          r <- t.takedownCountryReasons.toSeq.flatten
          reason <- TakedownLabels.fromWithholdingArm(r.takedownReason).toSeq
        } yield ((u.id, countryOf(reason)), reason.reasonType)
      }
      .group
      .reduce(higherPriority)
      .toTypedPipe

  private def countryOf(reason: TakedownLabels.Reason): String =
    reason.countryCode.getOrElse(TakedownLabels.WorldwideCopyrightCountryCode)

  private[under_the_hood] val TakedownsJsonField = "takedowns.asJson"

  private[under_the_hood] def parseTakedownsJson(json: String): Option[Takedowns] =
    Try(
      Takedowns.decode(
        new TJSONProtocol(new TMemoryInputTransport(json.getBytes(StandardCharsets.UTF_8))))
    ).toOption

  private[under_the_hood] def takedownsArms(t: Takedowns): Seq[(String, String)] =
    t.takedownCountryReasons.toSeq.flatten.flatMap { td =>
      TakedownLabels.fromWithholdingArm(td.takedownReason).map { reason =>
        (countryOf(reason), reason.reasonType.name)
      }
    }

  case class TakedownDiff(
    userId: Long,
    updatedAtMs: Long,
    before: Seq[(String, String)],
    after: Seq[(String, String)])

  private[under_the_hood] val TakedownsReasonsField = "takedowns.takedownCountryReasons"

  private val ArmPattern = """(\w+)\(\1\(([^()]*)\)\)""".r

  private[under_the_hood] def parseTakedownsToString(s: String): Seq[(String, String)] =
    ArmPattern
      .findAllMatchIn(s)
      .flatMap { m =>
        ReasonType.fromName(m.group(1)).collect {
          case t if t.countryScoped => (TakedownLabels.normalizeCountryCode(m.group(2)), t.name)
          case ReasonType.Dmca =>
            (TakedownLabels.WorldwideCopyrightCountryCode, ReasonType.Dmca.name)
        }
      }
      .toSeq
      .distinct

  private[under_the_hood] def takedownDiffs(
    mods: TypedPipe[UserModification],
    testUserIds: Set[Long]
  ): TypedPipe[TakedownDiff] =
    mods.flatMap { m =>
      m.userId match {
        case Some(userId) if !m.success.contains(false) && inScope(testUserIds, userId) =>
          val ts = m.updatedAtMsec.getOrElse(0L)
          m.update.getOrElse(Nil).flatMap { d =>
            d.fieldName match {
              case TakedownsReasonsField =>
                def arms(v: Option[String]) = v.toSeq.flatMap(parseTakedownsToString).distinct
                Some(TakedownDiff(userId, ts, arms(d.before), arms(d.after)))
              case TakedownsJsonField =>
                def arms(json: Option[String]) =
                  json.flatMap(parseTakedownsJson).toSeq.flatMap(takedownsArms).distinct
                Some(TakedownDiff(userId, ts, arms(d.before), arms(d.after)))
              case _ => None
            }
          }
        case _ => Nil
      }
    }

  type ArmEvent = (Long, Int, String)

  private[under_the_hood] def armEvents(
    diffs: TypedPipe[TakedownDiff]
  ): TypedPipe[((Long, String), List[ArmEvent])] =
    diffs
      .flatMap { d =>
        (d.before.map((_, 0)) ++ d.after.map((_, 1))).map {
          case ((country, arm), isAfter) => ((d.userId, country), (d.updatedAtMs, isAfter, arm))
        }
      }
      .distinct
      .group
      .toList
      .toTypedPipe

  private[under_the_hood] def armEvents(
    mods: TypedPipe[UserModification],
    testUserIds: Set[Long]
  ): TypedPipe[((Long, String), List[ArmEvent])] = armEvents(takedownDiffs(mods, testUserIds))

  private def armRank(arm: String): Int =
    ReasonType.fromName(arm).map(r => 9 - ReasonPriority.getOrElse(r, 9)).getOrElse(-1)

  private[under_the_hood] def deltas(
    pctdActions: TypedPipe[PctdAction],
    testUserIds: Set[Long],
    rangeStartMs: Long,
    pivotMs: Long
  ): TypedPipe[((Long, String), Delta)] =
    pctdActions.flatMap { a =>
      val eventMs = a.timestamp.toLong * 1000L
      if (a.`type` != PctdActionType.User || a.countryCode.trim.isEmpty ||
        !inScope(testUserIds, a.userId) || eventMs <= rangeStartMs || eventMs > pivotMs) None
      else
        Some(
          (
            (a.userId, a.countryCode.trim.toLowerCase(Locale.ROOT)),
            (
              eventMs,
              a.takendown,
              a.reason
                .flatMap(TakedownLabels.fromWithholdingArm).map(_.reasonType.name).getOrElse(""))))
    }.distinct

  case class KeyResolution(
    userId: Long,
    country: String,
    anchorReason: Option[ReasonType],
    armEvents: Seq[ArmEvent],
    auditReason: Option[ReasonType],
    deltas: Seq[Delta]) {

    def withheldAt(dayStartMs: Long): Boolean =
      deltas
        .find { case (eventMs, _, _) => eventMs > dayStartMs }
        .map { case (_, applied, _) => !applied }
        .getOrElse(anchorReason.isDefined)

    def armAt(dayStartMs: Long): Option[String] = {
      def pick(events: Seq[ArmEvent], latest: Boolean): Option[String] =
        if (events.isEmpty) None
        else {
          val ts = if (latest) events.map(_._1).max else events.map(_._1).min
          Some(events.collect { case (`ts`, _, arm) => arm }.maxBy(armRank))
        }
      pick(
        armEvents.filter { case (ts, isAfter, _) => isAfter == 1 && ts <= dayStartMs },
        latest = true)
        .orElse(
          pick(
            armEvents.filter { case (ts, isAfter, _) => isAfter == 0 && ts > dayStartMs },
            latest = false))
        .orElse(anchorReason.map(_.name))
        .orElse(auditReason.map(_.name))
    }

    def reasonAt(dayStartMs: Long): Option[ReasonType] =
      armAt(dayStartMs).flatMap(ReasonType.fromName)

    def reportedOverRange(dayStarts: Seq[Long]): String =
      dayStarts
        .filter(withheldAt)
        .map(t => reasonAt(t).fold("none")(_.name))
        .distinct
        .sorted
        .mkString(",")
  }

  private[under_the_hood] def resolveKeys(
    deltas: TypedPipe[((Long, String), Delta)],
    anchor: TypedPipe[((Long, String), ReasonType)],
    armEvents: TypedPipe[((Long, String), List[ArmEvent])],
    reducers: Int
  ): TypedPipe[KeyResolution] = {
    val grouped = deltas.group
    val joined = (if (reducers > 0) grouped.withReducers(reducers) else grouped).toList
      .outerJoin(anchor.group)
      .leftJoin(armEvents.group)
    joined.toTypedPipe.map {
      case ((userId, country), ((deltasOpt, anchorReason), events)) =>
        val sorted =
          deltasOpt.getOrElse(Nil).sortBy { case (eventMs, withheld, _) => (eventMs, withheld) }
        KeyResolution(
          userId,
          country,
          anchorReason,
          events.getOrElse(Nil).sortBy { case (ts, isAfter, arm) => (ts, isAfter, arm) },
          sorted
            .flatMap { case (_, _, name) => ReasonType.fromName(name) }.reduceOption(
              higherPriority),
          sorted
        )
    }
  }

  private[under_the_hood] def withheldAtDayStarts(
    resolutions: TypedPipe[KeyResolution],
    dayStarts: Seq[Long]
  ): TypedPipe[((Long, Int, String), String)] =
    resolutions.flatMap { k =>
      dayStarts.flatMap { dayStartMs =>
        if (!k.withheldAt(dayStartMs)) None
        else
          k.reasonAt(dayStartMs).map(r => ((k.userId, yyyymmddOf(dayStartMs), r.name), k.country))
      }
    }

  private[under_the_hood] def withheldAtDayStarts(
    deltas: TypedPipe[((Long, String), Delta)],
    anchor: TypedPipe[((Long, String), ReasonType)],
    armEvents: TypedPipe[((Long, String), List[ArmEvent])],
    dayStarts: Seq[Long],
    reducers: Int
  ): TypedPipe[((Long, Int, String), String)] =
    withheldAtDayStarts(resolveKeys(deltas, anchor, armEvents, reducers), dayStarts)

  private def armsTsv(arms: Seq[(String, String)]): String =
    arms.map { case (cc, arm) => s"$cc:$arm" }.sorted.mkString(",")

  private[under_the_hood] def keyDebugRow(
    k: KeyResolution,
    dayStarts: Seq[Long]
  ): (Long, String, String, String, String, String, String) =
    (
      k.userId,
      k.country,
      k.anchorReason.map(_.name).getOrElse(""),
      k.armEvents
        .map { case (ts, isAfter, arm) => s"$ts:${if (isAfter == 1) "after" else "before"}:$arm" }
        .mkString(","),
      k.auditReason.map(_.name).getOrElse(""),
      k.reportedOverRange(dayStarts),
      k.deltas
        .map {
          case (ms, applied, arm) =>
            s"$ms:${if (applied) "apply" else "reverse"}" + (if (arm.nonEmpty) s":$arm" else "")
        }.mkString(",")
    )

  private[under_the_hood] def diffDebugRow(d: TakedownDiff): (Long, Long, String, String) =
    (d.userId, d.updatedAtMs, armsTsv(d.before), armsTsv(d.after))

  private[under_the_hood] def accountLabelRows(
    withheld: TypedPipe[((Long, Int, String), String)],
    reducers: Int
  ): TypedPipe[UthDailyAccountLabel] = {
    val grouped = withheld.group
    (if (reducers > 0) grouped.withReducers(reducers) else grouped).toList.toTypedPipe.flatMap {
      case ((userId, day, reasonName), countries) =>
        ReasonType.fromName(reasonName).map { t =>
          UthDailyAccountLabel(
            userId = Some(userId),
            dayYyyymmdd = Some(day),
            label = Some(TakedownLabels.label(t, countries))
          )
        }
    }
  }
}

case class UthPctdAccountTakedownEventsConfig(
  testUserIds: Set[Long],
  reducers: Int,
  writeShards: Int,
  outputPath: String,
  debugPath: Option[String])

object UthPctdAccountTakedownEventsConfig {
  def fromArgs(args: Args): UthPctdAccountTakedownEventsConfig =
    UthPctdAccountTakedownEventsConfig(
      testUserIds = args
        .list("userIds")
        .flatMap(_.split(",")).map(_.trim).filter(_.nonEmpty).map(_.toLong).toSet,
      reducers = args.int("reducers", 50),
      writeShards = {
        val n = args.int("writeShards", 4)
        require(n > 0, s"--writeShards must be > 0; got $n")
        n
      },
      outputPath = args.required("outputPath"),
      debugPath = args.optional("debugPath")
    )
}

object UthPctdAccountTakedownEventsAdhoc
    extends UthPctdAccountTakedownEventsApp
    with TwitterExecutionApp {
  override def job: Execution[Unit] = Execution.withArgs { args =>
    implicit val dp: DateParser = DateParser.default
    runOnDateRange(
      DateRange.parse(args.list("date")),
      UthPctdAccountTakedownEventsConfig.fromArgs(args))
  }
}

object UthPctdAccountTakedownEventsProd
    extends UthPctdAccountTakedownEventsApp
    with TwitterScheduledExecutionApp {
  implicit val dp: DateParser = DateParser.default
  override def scheduledJob: Execution[Unit] = {
    val execArgs = AnalyticsBatchExecutionArgs(
      batchDesc = BatchDescription("uth_pctd_account_takedown_events_prod"),
      firstTime = BatchFirstTime(RichDate("2026-08-01")),
      batchIncrement = BatchIncrement(Days(1))
    )
    Execution.withArgs { args =>
      AnalyticsBatchExecution(execArgs) { dateRange =>
        runOnDateRange(dateRange, UthPctdAccountTakedownEventsConfig.fromArgs(args))
      }
    }
  }
}
