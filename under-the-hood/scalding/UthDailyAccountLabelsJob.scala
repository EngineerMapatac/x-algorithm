package com.twitter.visibility.under_the_hood

import com.twitter.algebird.Aggregator
import com.twitter.scalding.Args
import com.twitter.scalding.DateOps
import com.twitter.scalding.DateParser
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.Execution
import com.twitter.scalding.RichDate
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.scalding_internal.job.analytics_batch._
import com.twitter.usersource.snapshot.flat.UsersourceFlatScalaDataset
import com.twitter.visibility.under_the_hood.thriftscala.UthDailyAccountLabel
import com.twitter.visibility.user_labels.UserLabelRecordsScalaDataset
import java.util.TimeZone
import twadoop_config.configuration.log_categories.group.gizmoduck.GizmoduckUserModificationsScalaDataset

class UthDailyAccountLabelsApp {
  import UnderTheHoodCommon._
  import UthDailyAccountLabelsApp._

  implicit val tz: TimeZone = DateOps.UTC

  def runOnDateRange(dateRange: DateRange, config: UthDailyAccountLabelsConfig): Execution[Unit] = {
    val windowStartMs = dateRange.start.timestamp
    val windowEndMs = dateRange.end.timestamp + 1L
    require(windowStartMs % DayMs == 0, s"window start must be UTC midnight; got $windowStartMs")

    val targetDays: Seq[Long] = (windowStartMs until windowEndMs by DayMs).toSeq
    val replayFloorMs =
      windowStartMs - (config.seedLookbackDays + config.seedFreshnessSlackDays) * DayMs
    val seedRange = DateRange(dateRange.start - Days(config.seedLookbackDays), dateRange.start)

    val asOfExecution =
      loadSeed(seedRange)
        .map { case (_, _, createdAt, _) => createdAt }
        .groupAll
        .aggregate(Aggregator.max[Long])
        .toTypedPipe
        .map(_._2)
        .toIterableExecution

    asOfExecution.flatMap { asOfValues =>
      asOfValues.headOption match {
        case None =>
          Execution.failed(new IllegalStateException("empty user_label_records seed snapshot"))
        case Some(asOf) =>
          require(asOf <= windowStartMs, s"seed as-of $asOf after window start $windowStartMs")
          require(asOf >= replayFloorMs, s"seed as-of $asOf older than replay floor $replayFloorMs")

          val seedEvents =
            AccountLabelEvents.seedEventsPipe(
              loadSeed(seedRange),
              config.testUserIds,
              asOf
            )
          val gizmoduckMods =
            DAL
              .read(
                GizmoduckUserModificationsScalaDataset,
                DateRange(RichDate(replayFloorMs), dateRange.end)
              )
              .toTypedPipe

          val deltaEvents =
            if (config.seedOnly) TypedPipe.empty
            else
              AccountLabelEvents.deltaEventsPipe(
                gizmoduckMods,
                config.testUserIds,
                asOf,
                windowEndMs
              )

          val safetyFlagEvents =
            if (config.safetyFlagLabels.isEmpty) TypedPipe.empty
            else
              safetyFlagEventsPipe(
                DAL
                  .readMostRecentSnapshot(
                    UsersourceFlatScalaDataset,
                    DateRange(dateRange.start - Days(config.seedLookbackDays), dateRange.start)
                  )
                  .withColumns(GizmoduckSafetyFlags.UsersourceFlagColumns)
                  .toTypedPipe,
                if (config.seedOnly) TypedPipe.empty else gizmoduckMods,
                config.testUserIds,
                config.safetyFlagLabels,
                replayFloorMs,
                windowEndMs
              )

          val labelRows =
            labelsActiveOnDays(
              seedEvents ++ deltaEvents ++ safetyFlagEvents,
              targetDays,
              config.reducers)
              .map {
                case (userId, day, label) =>
                  UthDailyAccountLabel(Some(userId), Some(day), Some(label))
              }

          implicit val jobDr: DateRange = dateRange
          labelRows.writeDALExecution(
            UthDailyAccountLabelsScalaDataset,
            D.Daily,
            D.Suffix(s"${config.outputPath}/daily_account_labels"),
            D.Parquet
          )
      }
    }
  }

  private def loadSeed(
    seedRange: DateRange
  ): TypedPipe[(Long, String, Long, Long)] =
    DAL
      .readMostRecentSnapshot(UserLabelRecordsScalaDataset, seedRange)
      .withColumns(Set("user_id", "label_value", "created_at", "expires_at"))
      .toTypedPipe
      .map(r => (r.userId, r.labelValue, r.createdAt, r.expiresAt.getOrElse(Long.MaxValue)))
}

object UthDailyAccountLabelsApp {
  import UnderTheHoodCommon._
  type Event = AccountLabelEvents.Event

  private[under_the_hood] def safetyFlagEventsPipe(
    usersource: TypedPipe[com.twitter.usersource.snapshot.flat.thriftscala.FlatUser],
    gizmoduckMods: TypedPipe[com.twitter.gizmoduck.thriftscala.UserModification],
    testUserIds: Set[Long],
    flagLabels: Set[String],
    seedAsOfMs: Long,
    windowEndMs: Long
  ): TypedPipe[((Long, String), Event)] = {
    import GizmoduckSafetyFlags._
    val seed: TypedPipe[((Long, String), Event)] =
      usersource.flatMap { u =>
        u.id match {
          case Some(userId) if inScope(testUserIds, userId) =>
            usersourceFlags(u).collect {
              case (label, true) if flagLabels.contains(label) =>
                ((userId, label), (seedAsOfMs, Long.MaxValue, true))
            }
          case _ => Nil
        }
      }
    val deltas: TypedPipe[((Long, String), Event)] =
      flagDiffsPipe(gizmoduckMods, flagLabels, seedAsOfMs)
        .collect {
          case ((userId, label), (ts, _, after))
              if inScope(testUserIds, userId) && ts > seedAsOfMs && ts < windowEndMs =>
            ((userId, label), (ts, if (after) Long.MaxValue else 0L, after))
        }
    seed ++ deltas
  }

  private[under_the_hood] def labelsActiveOnDays(
    events: TypedPipe[((Long, String), Event)],
    targetDays: Seq[Long],
    reducers: Int
  ): TypedPipe[(Long, Int, String)] =
    applyReducers(events.group, reducers)
      .sortBy { case (timeMs, expiresMs, active) => (timeMs, active, expiresMs) }
      .mapValueStream { sortedEvents =>
        val buffered = sortedEvents.buffered
        var expiresAt = 0L
        var active = false
        var hasState = false
        targetDays.iterator.flatMap { dayStartMs =>
          while (buffered.hasNext && buffered.head._1 <= dayStartMs) {
            val event = buffered.next()
            expiresAt = event._2
            active = event._3
            hasState = true
          }
          if (hasState && active && expiresAt > dayStartMs) Some(yyyymmdd(dayStartMs))
          else None
        }
      }
      .toTypedPipe
      .map { case ((userId, label), day) => (userId, day, label) }
}

case class UthDailyAccountLabelsConfig(
  testUserIds: Set[Long],
  reducers: Int,
  seedOnly: Boolean,
  seedLookbackDays: Int,
  seedFreshnessSlackDays: Int,
  safetyFlagLabels: Set[String],
  outputPath: String)

object UthDailyAccountLabelsConfig {
  import GizmoduckSafetyFlags._

  private val KnownSafetyFlags =
    Map(
      "nsfwAdmin" -> NsfwAdminLabel
    )

  def fromArgs(args: Args): UthDailyAccountLabelsConfig = {
    val requestedFlags =
      args.list("safetyFlags").flatMap(_.split(",")).map(_.trim).filter(_.nonEmpty)
    val safetyFlagLabels: Set[String] =
      if (requestedFlags == Seq("none")) Set.empty
      else if (requestedFlags.isEmpty) Set(NsfwAdminLabel)
      else requestedFlags.flatMap(KnownSafetyFlags.get).toSet
    UthDailyAccountLabelsConfig(
      testUserIds = UnderTheHoodCommon.parseUserIds(args),
      reducers = args.int("reducers", 50),
      seedOnly = args.boolean("seedOnly"),
      seedLookbackDays = args.int("seedLookbackDays", 2),
      seedFreshnessSlackDays = args.int("seedFreshnessSlackDays", 2),
      safetyFlagLabels = safetyFlagLabels,
      outputPath = args.optional("outputPath").getOrElse("/user/<hadoop-role>/under_the_hood")
    )
  }
}

object UthDailyAccountLabelsAdhoc extends UthDailyAccountLabelsApp with TwitterExecutionApp {
  override def job: Execution[Unit] = Execution.withArgs { args =>
    runOnDateRange(UnderTheHoodDates.resolve(args), UthDailyAccountLabelsConfig.fromArgs(args))
  }
}

object UthDailyAccountLabelsProd
    extends UthDailyAccountLabelsApp
    with TwitterScheduledExecutionApp {
  implicit val dp: DateParser = DateParser.default
  override def scheduledJob: Execution[Unit] = {
    val execArgs = AnalyticsBatchExecutionArgs(
      batchDesc = BatchDescription("uth_daily_account_labels_prod"),
      firstTime = BatchFirstTime(RichDate("2026-06-30")),
      batchIncrement = BatchIncrement(Days(1))
    )
    Execution.withArgs { args =>
      AnalyticsBatchExecution(execArgs) { dateRange =>
        runOnDateRange(dateRange, UthDailyAccountLabelsConfig.fromArgs(args))
      }
    }
  }
}
