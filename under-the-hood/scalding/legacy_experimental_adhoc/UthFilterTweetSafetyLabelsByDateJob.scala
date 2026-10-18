package com.twitter.visibility.under_the_hood.legacy_experimental_adhoc

import com.twitter.visibility.under_the_hood.UnderTheHoodDates
import com.twitter.scalding.Args
import com.twitter.scalding.DateOps
import com.twitter.scalding.Days
import com.twitter.scalding.Execution
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.TypedTsv
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.AllowCrossClusterSameDC
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.spam.rtf.thriftscala.FlattenedSafetyLabel
import com.twitter.tweetsource.common.thriftscala.UnhydratedFlatTweet
import com.twitter.visibility.tweet_labels.TweetSafetyLabelsScalaDataset
import java.util.TimeZone
import tweetsource.common.UnhydratedFlatScalaDataset

class UthFilterTweetSafetyLabelsByDateApp {
  import UthFilterTweetSafetyLabelsByDateApp._

  implicit val tz: TimeZone = DateOps.UTC

  def runOnDateRange(
    dateRange: com.twitter.scalding.DateRange,
    config: UthFilterTweetSafetyLabelsByDateConfig
  ): Execution[Unit] = {
    val julyTweetIds: TypedPipe[(Long, Unit)] =
      loadTweetIdsInCreateWindow(
        DAL
          .read(UnhydratedFlatScalaDataset, dateRange)
          .withRemoteReadPolicy(AllowCrossClusterSameDC)
          .withColumns(Set("tweetId", "shareSourceTweetId", "nullcast"))
          .toTypedPipe,
        config.includeRetweetsAndNullcast
      )

    val snapshot: TypedPipe[(Long, (String, Option[Long], Option[Long]))] =
      loadSnapshotRows(
        DAL
          .readMostRecentSnapshotNoOlderThan(
            TweetSafetyLabelsScalaDataset,
            Days(config.snapshotMaxAgeDays)
          )
          .withRemoteReadPolicy(AllowCrossClusterSameDC)
          .withColumns(Set("tweet_id", "label_type", "created_at_msec", "expires_at_msec"))
          .toTypedPipe
      )

    val joined =
      if (config.reducers > 0) snapshot.join(julyTweetIds).withReducers(config.reducers)
      else snapshot.join(julyTweetIds)

    val out: TypedPipe[(Long, String, String, String)] =
      joined.map {
        case (tweetId, ((label, createdOpt, expiresOpt), _)) =>
          (
            tweetId,
            label,
            createdOpt.map(_.toString).getOrElse(""),
            expiresOpt.map(_.toString).getOrElse("")
          )
      }

    out
      .shard(config.writeShards)
      .writeExecution(TypedTsv[(Long, String, String, String)](config.outputPath))
  }
}

object UthFilterTweetSafetyLabelsByDateApp {

  private[under_the_hood] def loadTweetIdsInCreateWindow(
    tweets: TypedPipe[UnhydratedFlatTweet],
    includeRetweetsAndNullcast: Boolean
  ): TypedPipe[(Long, Unit)] =
    tweets.collect {
      case t if includeRetweetsAndNullcast || (t.shareSourceTweetId.isEmpty && !t.nullcast) =>
        (t.tweetId, ())
    }.distinct

  private[under_the_hood] def loadSnapshotRows(
    raw: TypedPipe[FlattenedSafetyLabel]
  ): TypedPipe[(Long, (String, Option[Long], Option[Long]))] =
    raw.map { f =>
      val name = f.labelType.originalName
      (f.tweetId, (name, f.createdAtMsec, f.expiresAtMsec))
    }
}

case class UthFilterTweetSafetyLabelsByDateConfig(
  includeRetweetsAndNullcast: Boolean,
  snapshotMaxAgeDays: Int,
  reducers: Int,
  writeShards: Int,
  outputPath: String)

object UthFilterTweetSafetyLabelsByDateConfig {
  def fromArgs(args: Args): UthFilterTweetSafetyLabelsByDateConfig =
    UthFilterTweetSafetyLabelsByDateConfig(
      includeRetweetsAndNullcast = args.boolean("includeRetweetsAndNullcast"),
      snapshotMaxAgeDays = args.int("snapshotMaxAgeDays", 5),
      reducers = args.int("reducers", 200),
      writeShards = {
        val n = args.int("writeShards", 50)
        require(n > 0, s"--writeShards must be > 0; got $n")
        n
      },
      outputPath = args
        .optional("outputPath")
        .getOrElse("/user/<hadoop-role>/under_the_hood/filtered_tweet_safety_labels")
    )
}

object UthFilterTweetSafetyLabelsByDateAdhoc
    extends UthFilterTweetSafetyLabelsByDateApp
    with TwitterExecutionApp {
  override def job: Execution[Unit] = Execution.withArgs { args =>
    runOnDateRange(
      UnderTheHoodDates.resolve(args),
      UthFilterTweetSafetyLabelsByDateConfig.fromArgs(args))
  }
}
