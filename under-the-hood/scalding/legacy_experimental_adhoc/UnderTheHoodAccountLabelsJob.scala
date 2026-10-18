package com.twitter.visibility.under_the_hood.legacy_experimental_adhoc

import com.twitter.visibility.under_the_hood.AccountLabelEvents
import com.twitter.visibility.under_the_hood.UnderTheHoodCommon
import com.twitter.visibility.under_the_hood.UnderTheHoodDates

import com.twitter.algebird.Aggregator
import com.twitter.gizmoduck.thriftscala.UserModification
import com.twitter.scalding.Args
import com.twitter.scalding.DateOps
import com.twitter.scalding.DateParser
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.Execution
import com.twitter.scalding.Months
import com.twitter.scalding.RichDate
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.TypedTsv
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.scalding_internal.job.analytics_batch._
import com.twitter.visibility.user_labels.UserLabelRecordsScalaDataset
import java.util.TimeZone
import twadoop_config.configuration.log_categories.group.gizmoduck.GizmoduckUserModificationsScalaDataset

class UnderTheHoodAccountLabelsApp {
  import UnderTheHoodAccountLabelsApp._

  implicit val tz: TimeZone = DateOps.UTC

  def runOnDateRange(dateRange: DateRange, config: UnderTheHoodAccountConfig): Execution[Unit] = {
    val monthStartMs: Long = dateRange.start.timestamp
    val windowEndMs: Long = dateRange.end.timestamp + 1L
    require(
      monthStartMs % UnderTheHoodCommon.DayMs == 0,
      s"window start must be day-aligned (UTC midnight); got $monthStartMs")
    val targetDays: Seq[Long] = {
      val firstDay =
        (dateRange.start.timestamp / UnderTheHoodCommon.DayMs) * UnderTheHoodCommon.DayMs
      (firstDay to dateRange.end.timestamp by UnderTheHoodCommon.DayMs).toSeq
    }
    val replayFloorMs: Long =
      monthStartMs - (config.seedLookbackDays + config.seedFreshnessSlackDays) * UnderTheHoodCommon.DayMs

    val seedRange: DateRange =
      DateRange(dateRange.start - Days(config.seedLookbackDays), dateRange.start)
    val seedProjected: TypedPipe[(Long, String, Long, Long)] =
      DAL
        .readMostRecentSnapshot(UserLabelRecordsScalaDataset, seedRange)
        .withColumns(Set("user_id", "label_value", "created_at", "expires_at"))
        .toTypedPipe
        .map(r => (r.userId, r.labelValue, r.createdAt, r.expiresAt.getOrElse(Long.MaxValue)))

    seedProjected.forceToDiskExecution.flatMap { seed =>
      val asOfExec: Execution[Iterable[Long]] =
        seed
          .map(_._3).groupAll.aggregate(Aggregator.max[Long]).toTypedPipe.map(
            _._2).toIterableExecution
      asOfExec.flatMap { asOfs =>
        asOfs.headOption match {
          case None =>
            Execution.failed(
              new IllegalStateException("seed snapshot is empty; cannot establish entering state"))
          case Some(asOf) =>
            require(
              asOf <= monthStartMs,
              s"seed AsOf ($asOf) is after window start ($monthStartMs) — future-dated created_at / clock skew")
            require(
              asOf >= replayFloorMs,
              s"seed AsOf ($asOf) older than replay floor ($replayFloorMs) — snapshot too stale; widen " +
                "seedLookbackDays or run events-only")

            val seedEvents = seedEventsPipe(seed, config.testUserIds, asOf)

            val deltaEvents: TypedPipe[((Long, String), Event)] =
              if (config.seedOnly) TypedPipe.empty
              else
                deltaEventsPipe(
                  DAL
                    .read(
                      GizmoduckUserModificationsScalaDataset,
                      DateRange(RichDate(replayFloorMs), dateRange.end))
                    .toTypedPipe,
                  config.testUserIds,
                  asOf,
                  windowEndMs
                )

            accountLabelsPipe(seedEvents ++ deltaEvents, targetDays, config.reducers)
              .writeExecution(
                TypedTsv[(Long, Int, String, Long)](config.outputPath + "/account_labels"))
        }
      }
    }
  }
}

object UnderTheHoodAccountLabelsApp {
  import UnderTheHoodCommon._

  type Event = AccountLabelEvents.Event

  private[under_the_hood] def seedEventsPipe(
    seed: TypedPipe[(Long, String, Long, Long)],
    testUserIds: Set[Long],
    asOfMs: Long
  ): TypedPipe[((Long, String), Event)] =
    AccountLabelEvents.seedEventsPipe(seed, testUserIds, asOfMs)

  private[under_the_hood] def deltaEventsPipe(
    raw: TypedPipe[UserModification],
    testUserIds: Set[Long],
    asOfMs: Long,
    windowEndMs: Long
  ): TypedPipe[((Long, String), Event)] =
    AccountLabelEvents.deltaEventsPipe(raw, testUserIds, asOfMs, windowEndMs)

  private[under_the_hood] def accountLabelsPipe(
    events: TypedPipe[((Long, String), Event)],
    targetDays: Seq[Long],
    reducers: Int
  ): TypedPipe[(Long, Int, String, Long)] =
    applyReducers(events.group, reducers)
      .sortBy { case (timeMs, expiresMs, active) => (timeMs, active, expiresMs) }
      .mapValueStream { sortedEvents =>
        val evIt = sortedEvents.buffered
        val daysByMonth = scala.collection.mutable.LinkedHashMap.empty[Int, Long]
        var curExpires = 0L
        var curActive = false
        var haveCur = false
        targetDays.foreach { d =>
          while (evIt.hasNext && evIt.head._1 <= d) {
            val e = evIt.next(); curExpires = e._2; curActive = e._3; haveCur = true
          }
          if (haveCur && curActive && curExpires > d) {
            val ym = UnderTheHoodCommon.yyyymm(d)
            daysByMonth.update(ym, daysByMonth.getOrElse(ym, 0L) + 1L)
          }
        }
        daysByMonth.iterator
      }
      .toTypedPipe
      .collect { case ((userId, label), (ym, days)) if days > 0 => (userId, ym, label, days) }
}

case class UnderTheHoodAccountConfig(
  testUserIds: Set[Long],
  reducers: Int,
  seedOnly: Boolean,
  seedLookbackDays: Int,
  seedFreshnessSlackDays: Int,
  outputPath: String)

object UnderTheHoodAccountConfig {
  def fromArgs(args: Args): UnderTheHoodAccountConfig =
    UnderTheHoodAccountConfig(
      testUserIds = UnderTheHoodCommon.parseUserIds(args),
      reducers = args.int("reducers", 50),
      seedOnly = args.boolean("seedOnly"),
      seedLookbackDays = args.int("seedLookbackDays", 2),
      seedFreshnessSlackDays = args.int("seedFreshnessSlackDays", 2),
      outputPath =
        args.optional("outputPath").getOrElse(s"/user/${sys.env("USER")}/uth_account_labels")
    )
}

object UnderTheHoodAccountLabelsAdhoc
    extends UnderTheHoodAccountLabelsApp
    with TwitterExecutionApp {
  override def job: Execution[Unit] = Execution.withArgs { args =>
    runOnDateRange(UnderTheHoodDates.resolve(args), UnderTheHoodAccountConfig.fromArgs(args))
  }
}

object UnderTheHoodAccountLabelsProd
    extends UnderTheHoodAccountLabelsApp
    with TwitterScheduledExecutionApp {
  implicit val dp: DateParser = DateParser.default

  override def scheduledJob: Execution[Unit] = {
    val execArgs = AnalyticsBatchExecutionArgs(
      batchDesc = BatchDescription("under_the_hood_account_labels_prod"),
      firstTime = BatchFirstTime(RichDate("2026-07-01")),
      batchIncrement = BatchIncrement(Months(1))
    )
    Execution.withArgs { args =>
      AnalyticsBatchExecution(execArgs) { dateRange =>
        runOnDateRange(dateRange, UnderTheHoodAccountConfig.fromArgs(args))
      }
    }
  }
}
