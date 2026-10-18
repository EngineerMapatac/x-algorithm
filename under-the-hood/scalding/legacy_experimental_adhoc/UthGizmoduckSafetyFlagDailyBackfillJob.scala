package com.twitter.visibility.under_the_hood.legacy_experimental_adhoc

import com.twitter.gizmoduck.thriftscala.UserModification
import com.twitter.scalding.Args
import com.twitter.scalding.DateOps
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.Execution
import com.twitter.scalding.RichDate
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.TypedTsv
import com.twitter.scalding.parquet.scrooge.DailySuffixParquetScrooge
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.usersource.snapshot.flat.UsersourceFlatScalaDataset
import com.twitter.visibility.user_labels.UserLabelRecordsScalaDataset
import com.twitter.visibility.under_the_hood.GizmoduckLabelDiff
import com.twitter.visibility.under_the_hood.GizmoduckSafetyFlags
import com.twitter.visibility.under_the_hood.UnderTheHoodCommon
import com.twitter.visibility.under_the_hood.UnderTheHoodDates
import com.twitter.visibility.under_the_hood.UthDailyAccountLabelsScalaDataset
import com.twitter.visibility.under_the_hood.thriftscala.UthDailyAccountLabel
import java.util.TimeZone

class UthGizmoduckSafetyFlagDailyBackfillApp {
  import GizmoduckSafetyFlags._
  import UnderTheHoodCommon._
  import UthGizmoduckSafetyFlagDailyBackfillApp._

  implicit val tz: TimeZone = DateOps.UTC

  def runOnDateRange(
    dateRange: DateRange,
    config: UthGizmoduckSafetyFlagDailyBackfillConfig
  ): Execution[Unit] = {
    val windowStartMs = dateRange.start.timestamp
    val windowEndMs = dateRange.end.timestamp + 1L
    require(windowStartMs % DayMs == 0, s"window start must be UTC midnight; got $windowStartMs")
    require(
      !(config.recomputeLabelsFromScratch && config.existingRowsPath.nonEmpty),
      "--recomputeLabelsFromScratch and --existingRowsPath are mutually exclusive")
    require(
      !config.recomputeLabelsFromScratch || config.writeDailyAccountLabels,
      "--recomputeLabelsFromScratch requires --writeDailyAccountLabels")
    val targetDays: Seq[Long] =
      (windowStartMs until windowEndMs by DayMs).toSeq

    val anchorDay: RichDate = config.anchorDate
    require(
      anchorDay.timestamp >= dateRange.end.timestamp,
      s"--anchorDate ($anchorDay) must be at/after the window end (${dateRange.end}): the " +
        "reconstruction anchors on a later cut and walks diffs backwards"
    )

    val anchorRange = DateRange(anchorDay - Days(config.anchorLookbackDays), anchorDay)
    val anchor: TypedPipe[(Long, Map[String, Boolean])] =
      DAL
        .readMostRecentSnapshot(UsersourceFlatScalaDataset, anchorRange)
        .withColumns(UsersourceFlagColumns)
        .toTypedPipe
        .flatMap { u =>
          u.id.flatMap { userId =>
            val flags = usersourceFlags(u).filter { case (k, _) => config.flagLabels.contains(k) }
            if (flags.valuesIterator.exists(identity)) Some((userId, flags)) else None
          }
        }

    val diffs: TypedPipe[((Long, String), FlagDiff)] =
      flagDiffsPipe(
        DAL
          .read(
            GizmoduckUserModificationsScalaDatasetAlias,
            DateRange(RichDate(windowStartMs), anchorDay)
          )
          .toTypedPipe,
        config.flagLabels,
        windowStartMs
      )

    val rows: TypedPipe[(Long, Int, String)] =
      dailyFlagRows(anchor, diffs, targetDays, config.flagLabels, config.reducers)

    if (!config.writeDailyAccountLabels) {
      rows.forceToDiskExecution.flatMap { pipe =>
        requireNonEmpty(pipe.map(_ => 1L), "flag reconstruction (TSV mode)") {
          pipe.writeExecution(
            TypedTsv[(Long, Int, String)](s"${config.outputPath}/safety_flag_daily"))
        }
      }
    } else {
      val existing: TypedPipe[UthDailyAccountLabel] =
        if (config.recomputeLabelsFromScratch) {
          val labelAnchor: TypedPipe[((Long, String), Long)] =
            DAL
              .readMostRecentSnapshot(
                UserLabelRecordsScalaDataset,
                DateRange(anchorDay - Days(config.anchorLookbackDays), anchorDay))
              .withColumns(Set("user_id", "label_value", "expires_at"))
              .toTypedPipe
              .map(r => ((r.userId, r.labelValue), r.expiresAt.getOrElse(Long.MaxValue)))
          val labelDiffs: TypedPipe[((Long, String), PresenceDiff)] =
            labelDiffsPipe(
              DAL
                .read(
                  GizmoduckUserModificationsScalaDatasetAlias,
                  DateRange(RichDate(windowStartMs), anchorDay))
                .toTypedPipe,
              windowStartMs
            )
          dailyPresenceRows(labelAnchor, labelDiffs, targetDays, config.reducers)
            .map {
              case (userId, day, label) =>
                UthDailyAccountLabel(Some(userId), Some(day), Some(label))
            }
        } else
          config.existingRowsPath match {
            case Some(path) =>
              TypedPipe.from(new DailySuffixParquetScrooge[UthDailyAccountLabel](path, dateRange))
            case None =>
              DAL.read(UthDailyAccountLabelsScalaDataset, dateRange).toTypedPipe
          }
      rows.forceToDiskExecution.flatMap { flagPipe =>
        requireNonEmpty(flagPipe.map(_ => 1L), "flag reconstruction (NsfwAdmin rows)") {
          val flagRows: TypedPipe[UthDailyAccountLabel] =
            flagPipe.map {
              case (userId, day, label) =>
                UthDailyAccountLabel(Some(userId), Some(day), Some(label))
            }
          val combined = applyReducers(
            (existing ++ flagRows)
              .map(r => ((r.userId, r.dayYyyymmdd, r.label), ()))
              .group,
            config.reducers
          ).sum.keys.map { case (userId, day, label) => UthDailyAccountLabel(userId, day, label) }

          combined.forceToDiskExecution.flatMap { pipe =>
            requireNonEmpty(pipe.map(_ => 1L), "combined existing∪flag rows") {
              Execution
                .sequence(targetDays.map { dayStartMs =>
                  val day = yyyymmdd(dayStartMs)
                  implicit val partitionRange: DateRange =
                    DateRange(RichDate(dayStartMs), RichDate(dayStartMs + DayMs - 1L))
                  pipe
                    .filter(_.dayYyyymmdd.contains(day))
                    .shard(config.writeShards)
                    .writeDALExecution(
                      UthDailyAccountLabelsScalaDataset,
                      D.Daily,
                      D.Suffix(s"${config.outputPath}/daily_account_labels"),
                      D.Parquet
                    )
                }).unit
            }
          }
        }
      }
    }
  }
}

object UthGizmoduckSafetyFlagDailyBackfillApp {
  import GizmoduckSafetyFlags._
  import UnderTheHoodCommon._

  private[legacy_experimental_adhoc] val GizmoduckUserModificationsScalaDatasetAlias =
    twadoop_config.configuration.log_categories.group.gizmoduck.GizmoduckUserModificationsScalaDataset

  private[legacy_experimental_adhoc] def requireNonEmpty(
    ones: TypedPipe[Long],
    what: String
  )(
    write: => Execution[Unit]
  ): Execution[Unit] =
    ones.sum.toOptionExecution.flatMap {
      case None | Some(0L) =>
        Execution.failed(
          new IllegalStateException(
            s"$what produced 0 rows — refusing to write empty output. " +
              "If this is DAL mode: empty day dirs at the existing-rows source read as empty " +
              "(not missing); point --existingRowsPath at the PRE_* backup with real parquet, " +
              "or use --recomputeLabelsFromScratch. Also confirm --anchorDate has a landed " +
              "usersource cut and gizmoduck mods cover [windowStart, anchorDate]."))
      case Some(_) => write
    }

  type PresenceDiff = (Long, Option[Long], Option[Long])

  private[legacy_experimental_adhoc] def labelDiffsPipe(
    raw: TypedPipe[UserModification],
    minTsMs: Long
  ): TypedPipe[((Long, String), PresenceDiff)] =
    raw.flatMap { m =>
      val ts = m.updatedAtMsec.getOrElse(0L)
      if (m.userId.isEmpty || m.success.contains(false) || ts < minTsMs)
        Iterable.empty[((Long, String), PresenceDiff)]
      else {
        val userId = m.userId.get
        m.update
          .getOrElse(Nil).iterator.filter(_.fieldName == "labels.asJson").flatMap { d =>
            val decoded = for {
              beforeStr <- d.before
              afterStr <- d.after
              before <- GizmoduckLabelDiff.labelApplyAndExpireTimes(beforeStr)
              after <- GizmoduckLabelDiff.labelApplyAndExpireTimes(afterStr)
            } yield {
              (before.keySet ++ after.keySet).map { label =>
                ((userId, label), (ts, before.get(label).map(_._2), after.get(label).map(_._2)))
              }
            }
            decoded.getOrElse(Set.empty[((Long, String), PresenceDiff)])
          }.toSeq
      }
    }

  private[legacy_experimental_adhoc] def dailyPresenceRows(
    anchor: TypedPipe[((Long, String), Long)],
    diffs: TypedPipe[((Long, String), PresenceDiff)],
    targetDays: Seq[Long],
    reducers: Int
  ): TypedPipe[(Long, Int, String)] = {
    val grouped = applyReducers(
      anchor
        .map { case (k, exp) => (k, Left(exp): Either[Long, PresenceDiff]) }
        .++(diffs.map { case (k, d) => (k, Right(d): Either[Long, PresenceDiff]) })
        .group,
      reducers
    )
    grouped.toList.flatMap {
      case ((userId, label), entries) =>
        val anchorState: Option[Long] = entries.collectFirst { case Left(exp) => exp }
        val ds = entries.collect { case Right(d) => d }.sortBy(_._1)
        targetDays.flatMap { dayStartMs =>
          val atOrBefore = ds.filter(_._1 <= dayStartMs)
          val state: Option[Long] =
            if (atOrBefore.nonEmpty) atOrBefore.last._3
            else
              ds.headOption match {
                case Some((_, before, _)) => before
                case None => anchorState
              }
          if (state.exists(_ > dayStartMs)) Some((userId, yyyymmdd(dayStartMs), label))
          else None
        }
    }
  }

  private[legacy_experimental_adhoc] def dailyFlagRows(
    anchor: TypedPipe[(Long, Map[String, Boolean])],
    diffs: TypedPipe[((Long, String), FlagDiff)],
    targetDays: Seq[Long],
    flagLabels: Set[String],
    reducers: Int
  ): TypedPipe[(Long, Int, String)] = {
    val anchorEvents: TypedPipe[((Long, String), Option[Boolean])] =
      anchor.flatMap {
        case (userId, flags) =>
          flags.collect { case (label, v) if v => ((userId, label), Some(v)) }
      }
    val diffEvents: TypedPipe[((Long, String), FlagDiff)] = diffs

    val grouped = applyReducers(
      anchorEvents
        .map { case (k, v) => (k, Left(v): Either[Option[Boolean], FlagDiff]) }
        .++(diffEvents.map { case (k, d) => (k, Right(d): Either[Option[Boolean], FlagDiff]) })
        .group,
      reducers
    )

    grouped.toList.flatMap {
      case ((userId, label), entries) =>
        val anchorTrue = entries.exists(_.isLeft)
        val ds = entries.collect { case Right(d) => d }.sortBy(_._1)
        targetDays.flatMap { dayStartMs =>
          val atOrBefore = ds.filter(_._1 <= dayStartMs)
          val state: Boolean =
            if (atOrBefore.nonEmpty) atOrBefore.last._3
            else
              ds.headOption match {
                case Some((_, before, _)) => before
                case None => anchorTrue
              }
          if (state) Some((userId, yyyymmdd(dayStartMs), label)) else None
        }
    }
  }
}

case class UthGizmoduckSafetyFlagDailyBackfillConfig(
  flagLabels: Set[String],
  reducers: Int,
  anchorDate: RichDate,
  anchorLookbackDays: Int,
  writeDailyAccountLabels: Boolean,
  existingRowsPath: Option[String],
  recomputeLabelsFromScratch: Boolean,
  writeShards: Int,
  outputPath: String)

object UthGizmoduckSafetyFlagDailyBackfillConfig {

  private val KnownFlags =
    Map(
      "nsfwAdmin" -> GizmoduckSafetyFlags.NsfwAdminLabel
    )

  def fromArgs(args: Args): UthGizmoduckSafetyFlagDailyBackfillConfig = {
    implicit val tz: TimeZone = DateOps.UTC
    implicit val dp: com.twitter.scalding.DateParser = com.twitter.scalding.DateParser.default
    val requested = args.list("flags") match {
      case Nil => Seq("nsfwAdmin")
      case fs => fs.flatMap(_.split(",")).map(_.trim).filter(_.nonEmpty)
    }
    val anchorDateStr = args
      .optional("anchorDate")
      .getOrElse(throw new IllegalArgumentException(
        "--anchorDate YYYY-MM-DD is required (usersource cut day to anchor on; must be >= window end)"))
    UthGizmoduckSafetyFlagDailyBackfillConfig(
      flagLabels = requested.flatMap(KnownFlags.get).toSet,
      reducers = args.int("reducers", 5000),
      anchorDate = RichDate(anchorDateStr),
      anchorLookbackDays = args.int("anchorLookbackDays", 3),
      writeDailyAccountLabels = args.boolean("writeDailyAccountLabels"),
      existingRowsPath = args.optional("existingRowsPath"),
      recomputeLabelsFromScratch = args.boolean("recomputeLabelsFromScratch"),
      writeShards = {
        val n = args.int("writeShards", 50)
        require(n > 0, s"--writeShards must be > 0; got $n")
        n
      },
      outputPath = args
        .optional("outputPath")
        .getOrElse(
          if (args.boolean("writeDailyAccountLabels"))
            "/user/<hadoop-role>/under_the_hood"
          else
            "/user/<hadoop-role>/under_the_hood/experimental_safety_flag_daily")
    )
  }
}

object UthGizmoduckSafetyFlagDailyBackfillAdhoc
    extends UthGizmoduckSafetyFlagDailyBackfillApp
    with TwitterExecutionApp {
  override def job: Execution[Unit] = Execution.withArgs { args =>
    runOnDateRange(
      UnderTheHoodDates.resolve(args),
      UthGizmoduckSafetyFlagDailyBackfillConfig.fromArgs(args))
  }
}
