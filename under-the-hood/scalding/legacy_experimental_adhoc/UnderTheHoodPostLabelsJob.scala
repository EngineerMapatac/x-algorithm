package com.twitter.visibility.under_the_hood.legacy_experimental_adhoc

import com.twitter.visibility.under_the_hood.UnderTheHoodCommon
import com.twitter.visibility.under_the_hood.UnderTheHoodDates
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
import com.twitter.spam.rtf.thriftscala.ActionType
import com.twitter.spam.rtf.thriftscala.FlattenedSafetyLabel
import com.twitter.spam.rtf.thriftscala.TweetSafetyLabelEvent
import com.twitter.tweetsource.common.thriftscala.UnhydratedFlatTweet
import com.twitter.visibility.tweet_labels.TweetSafetyLabelsScalaDataset
import java.util.TimeZone
import tweetsource.common.UnhydratedFlatScalaDataset
import twadoop_config.configuration.log_categories.group.visibility.TweetSafetyLabelEventsScalaDataset

class UnderTheHoodPostLabelsApp {
  import UnderTheHoodPostLabelsApp._

  implicit val tz: TimeZone = DateOps.UTC

  def runOnDateRange(dateRange: DateRange, config: UnderTheHoodPostConfig): Execution[Unit] = {
    val monthEndMs: Long = dateRange.end.timestamp + 1L
    val monthStartMs: Long = dateRange.start.timestamp

    val postBase = postBasePipe(
      DAL
        .read(UnhydratedFlatScalaDataset, dateRange)
        .withColumns(Set("userId", "tweetId", "initial_tweet_id", "shareSourceTweetId", "nullcast"))
        .toTypedPipe,
      config.testUserIds,
      monthStartMs,
      monthEndMs
    )

    val postTweetInfo =
      postBase.map { case (tid, uid, ym, logical) => (tid, (logical, uid, ym)) }.distinct

    val eventRows = eventRowsPipe(
      DAL
        .read(TweetSafetyLabelEventsScalaDataset, DateRange(dateRange.start, dateRange.end))
        .toTypedPipe,
      monthStartMs,
      monthEndMs)

    val snapshotRows: TypedPipe[(Long, (String, Long, Boolean))] =
      if (!config.snapshotUnion) TypedPipe.empty
      else
        snapshotRowsPipe(
          DAL
            .readMostRecentSnapshot(
              TweetSafetyLabelsScalaDataset,
              DateRange(dateRange.end - Days(config.snapshotMaxAgeDays), dateRange.end))
            .withColumns(Set("tweet_id", "label_type", "created_at_msec", "expires_at_msec"))
            .toTypedPipe,
          monthStartMs,
          monthEndMs
        )

    val denominator = denominatorPipe(postBase, config.reducers)
    val postLabels = postLabelsPipe(eventRows, snapshotRows, postTweetInfo, config)

    Execution
      .zip(
        denominator.writeExecution(TypedTsv[(Long, Int, Long)](config.outputPath + "/denominator")),
        postLabels.writeExecution(
          TypedTsv[(Long, Int, String, Long, Long)](config.outputPath + "/post_labels"))
      ).unit
  }
}

object UnderTheHoodPostLabelsApp {
  import UnderTheHoodCommon._

  private val TwitterEpochMs: Long = 1288834974657L
  private[under_the_hood] def snowflakeCreatedMs(id: Long): Long = (id >> 22) + TwitterEpochMs

  private[under_the_hood] def postBasePipe(
    raw: TypedPipe[UnhydratedFlatTweet],
    testUserIds: Set[Long],
    monthStartMs: Long,
    monthEndMs: Long
  ): TypedPipe[(Long, Long, Int, Long)] =
    raw.flatMap { t =>
      if (inScope(testUserIds, t.userId) && t.shareSourceTweetId.isEmpty && !t.nullcast) {
        val logical = t.initialTweetId.getOrElse(t.tweetId)
        val createdMs = snowflakeCreatedMs(logical)
        if (createdMs >= monthStartMs && createdMs < monthEndMs)
          Some((t.tweetId, t.userId, yyyymm(createdMs), logical))
        else None
      } else None
    }

  private[under_the_hood] def denominatorPipe(
    postBase: TypedPipe[(Long, Long, Int, Long)],
    reducers: Int
  ): TypedPipe[(Long, Int, Long)] =
    applyReducers(
      postBase
        .map { case (_, uid, ym, logical) => (uid, ym, logical) }
        .distinct
        .map { case (uid, ym, _) => ((uid, ym), 1L) }
        .group,
      reducers).sum.toTypedPipe
      .map { case ((uid, ym), n) => (uid, ym, n) }

  private[under_the_hood] def eventRowsPipe(
    raw: TypedPipe[TweetSafetyLabelEvent],
    monthStartMs: Long,
    monthEndMs: Long
  ): TypedPipe[(Long, (String, Long, Boolean))] =
    raw.flatMap { e =>
      val name = e.labelType.originalName
      val isApply = e.actionType == ActionType.Applied
      val isRemove = e.actionType == ActionType.Removed
      val ts = e.timestampMs.orElse(e.label.flatMap(_.createdAtMsec)).getOrElse(0L)
      val keep = (isApply || isRemove) && ts >= monthStartMs && ts < monthEndMs
      if (keep) Some((e.tweetId, (name, ts, isApply))) else None
    }

  private[under_the_hood] def snapshotRowsPipe(
    raw: TypedPipe[FlattenedSafetyLabel],
    monthStartMs: Long,
    monthEndMs: Long
  ): TypedPipe[(Long, (String, Long, Boolean))] =
    raw.flatMap { f =>
      val name = f.labelType.originalName
      val expires = f.expiresAtMsec.getOrElse(Long.MaxValue)
      if (!(expires > monthStartMs)) None
      else
        f.createdAtMsec match {
          case Some(created) =>
            if (created < monthEndMs) Some((f.tweetId, (name, created, true))) else None
          case None => Some((f.tweetId, (name, Long.MinValue, true)))
        }
    }

  private[under_the_hood] def postLabelsPipe(
    eventRows: TypedPipe[(Long, (String, Long, Boolean))],
    snapshotRows: TypedPipe[(Long, (String, Long, Boolean))],
    postTweetInfo: TypedPipe[(Long, (Long, Long, Int))],
    config: UnderTheHoodPostConfig
  ): TypedPipe[(Long, Int, String, Long, Long)] = {
    val postTweetIds = postTweetInfo.map { case (tid, _) => (tid, ()) }.distinct.group

    def restrictToPosts[V](p: TypedPipe[(Long, V)]): TypedPipe[(Long, V)] =
      if (config.testUserIds.isEmpty) p
      else p.hashJoin(postTweetIds).map { case (tid, (v, _)) => (tid, v) }

    val allRows = restrictToPosts(eventRows) ++ restrictToPosts(snapshotRows)

    val normalized = {
      val joined =
        if (config.reducers > 0) allRows.join(postTweetInfo).withReducers(config.reducers)
        else allRows.join(postTweetInfo)
      joined.map {
        case (_, ((name, ts, active), (logical, uid, ym))) =>
          ((logical, uid, ym, name), (active, ts, active))
      }
    }

    val reduced = applyReducers(normalized.group, config.reducers).reduce { (a, b) =>
      val everActive = a._1 || b._1
      val (lastTs, lastActive) =
        if (a._2 != b._2) {
          if (a._2 > b._2) (a._2, a._3) else (b._2, b._3)
        } else
          (a._2, a._3 || b._3)
      (everActive, lastTs, lastActive)
    }.toTypedPipe

    applyReducers(
      reduced.collect {
        case ((_, uid, ym, name), (everActive, _, lastActive)) if everActive =>
          ((uid, ym, name), (1L, if (lastActive) 0L else 1L))
      }.group,
      config.reducers).sum.toTypedPipe
      .map {
        case ((userId, ym, label), (carried, removed)) => (userId, ym, label, carried, removed)
      }
  }
}

case class UnderTheHoodPostConfig(
  testUserIds: Set[Long],
  reducers: Int,
  snapshotUnion: Boolean,
  snapshotMaxAgeDays: Int,
  outputPath: String)

object UnderTheHoodPostConfig {
  def fromArgs(args: Args): UnderTheHoodPostConfig =
    UnderTheHoodPostConfig(
      testUserIds = UnderTheHoodCommon.parseUserIds(args),
      reducers = args.int("reducers", 50),
      snapshotUnion = args.boolean("snapshotUnion"),
      snapshotMaxAgeDays = args.int("snapshotMaxAgeDays", 10),
      outputPath =
        args.optional("outputPath").getOrElse(s"/user/${sys.env("USER")}/uth_post_labels")
    )
}

object UnderTheHoodPostLabelsAdhoc extends UnderTheHoodPostLabelsApp with TwitterExecutionApp {
  override def job: Execution[Unit] = Execution.withArgs { args =>
    runOnDateRange(UnderTheHoodDates.resolve(args), UnderTheHoodPostConfig.fromArgs(args))
  }
}

object UnderTheHoodPostLabelsProd
    extends UnderTheHoodPostLabelsApp
    with TwitterScheduledExecutionApp {
  implicit val dp: DateParser = DateParser.default

  override def scheduledJob: Execution[Unit] = {
    val execArgs = AnalyticsBatchExecutionArgs(
      batchDesc = BatchDescription("under_the_hood_post_labels_prod"),
      firstTime = BatchFirstTime(RichDate("2026-07-01")),
      batchIncrement = BatchIncrement(Months(1))
    )
    Execution.withArgs { args =>
      AnalyticsBatchExecution(execArgs) { dateRange =>
        runOnDateRange(dateRange, UnderTheHoodPostConfig.fromArgs(args))
      }
    }
  }
}
