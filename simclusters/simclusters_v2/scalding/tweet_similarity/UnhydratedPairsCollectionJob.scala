package com.twitter.simclusters_v2.scalding.tweet_similarity

import com.twitter.ads.dataservice_account.snapshot.jobs.DbSnapshotsPromotedTweetsScalaDataset
import com.twitter.conversions.DurationOps._
import com.twitter.dal.client.dataset.TimePartitionedDALDataset
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcrevAtla
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.simclusters_v2.scalding.tweet_similarity.TweetPairLabelCollectionUtil.MaxFavPerUser
import com.twitter.simclusters_v2.thriftscala.LabelledTweetPairs
import com.twitter.simclusters_v2.thriftscala.{FeaturedTweet => FeaturedTweetThrift}
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import java.util.TimeZone

object UnhydratedPairsCollectionJob {
  val LookbackDays = 2

  def getLabelledPairs(
    dateRange: DateRange,
    timeframe: Long,
    maxSamplesPerClass: Int,
    dalDataset: TimePartitionedDALDataset[LabelledTweetPairs],
    outputPath: String
  )(
    implicit timeZone: TimeZone
  ): Execution[Unit] = {

    val promotedTweets = DAL
      .readMostRecentSnapshot(DbSnapshotsPromotedTweetsScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcrevAtla))
      .toTypedPipe

    val tweetAuthorPairs =
      TweetPairLabelCollectionUtil.getTweetAuthorPairs(dateRange.prepend(Days(LookbackDays)))

    val tweets =
      TweetPairLabelCollectionUtil.getNonPromotedTweets(promotedTweets, tweetAuthorPairs.keys)

    val coengagedPairs = TweetPairLabelCollectionUtil.getCoengagedPairs(
      TweetPairLabelCollectionUtil.getFavEvents(dateRange, MaxFavPerUser),
      tweets,
      timeframe)

    val engagedTweets = coengagedPairs.map {
      case (_, queryFeaturedTweet, _, _) => queryFeaturedTweet.tweet
    }.distinct

    val coimpressedPairs = TweetPairLabelCollectionUtil
      .getCoimpressedPairs(
        TweetPairLabelCollectionUtil.getImpressionEvents(dateRange),
        engagedTweets,
        timeframe)

    val rawLabelledPairs =
      TweetPairLabelCollectionUtil.computeLabelledTweetPairs(coengagedPairs, coimpressedPairs)

    val labelledPairs =
      if (maxSamplesPerClass > 0)
        TweetPairLabelCollectionUtil.getQueryTweetBalancedClassPairs(
          rawLabelledPairs,
          maxSamplesPerClass)
      else
        rawLabelledPairs

    val perQueryStatsExec =
      if (maxSamplesPerClass > 0) {
        Execution
          .zip(
            TweetPairLabelCollectionUtil
              .getPerQueryStatsExec(rawLabelledPairs, s"$outputPath/per_query_stats", "raw"),
            TweetPairLabelCollectionUtil
              .getPerQueryStatsExec(labelledPairs, s"$outputPath/per_query_stats", "final")
          ).unit
      } else {
        TweetPairLabelCollectionUtil.getPerQueryStatsExec(
          labelledPairs,
          s"$outputPath/per_query_stats",
          "final")
      }

    Execution
      .zip(
        labelledPairs
          .map {
            case (queryFeaturedTweet, candidateFeaturedTweet, label) =>
              LabelledTweetPairs(
                FeaturedTweetThrift(
                  tweetId = queryFeaturedTweet.tweet,
                  timestamp = queryFeaturedTweet.timestamp),
                FeaturedTweetThrift(
                  tweetId = candidateFeaturedTweet.tweet,
                  timestamp = candidateFeaturedTweet.timestamp),
                label
              )
          }
          .writeDALExecution(dalDataset, D.Daily, D.Suffix(outputPath), D.EBLzo())(dateRange),
        perQueryStatsExec
      ).unit
  }
}

object UnhydratedPairsCollectionAdhocApp extends TwitterExecutionApp {
  implicit val timeZone: TimeZone = DateOps.UTC
  implicit val dateParser: DateParser = DateParser.default

  override def job: Execution[Unit] =
    Execution.withId { implicit uniqueId =>
      Execution.withArgs { args: Args =>
        implicit val dateRange: DateRange = DateRange.parse(args.list("date"))
        val maxSamplesPerClass: Int = args.int("samples_per_query_tweet_class", default = 2000)
        val timeframe: Int = 30
        val outputPath: String = s"${args("output_path")}_${timeframe}min"

        UnhydratedPairsCollectionJob.getLabelledPairs(
          dateRange,
          timeframe.minute.inMilliseconds,
          maxSamplesPerClass,
          TweetSimilarityUnhydratedPairs30MinScalaDataset,
          outputPath
        )
      }
    }
}

object UnhydratedPairsCollection30MinScheduledApp extends ScheduledExecutionApp {

  override def batchIncrement: Duration = Hours(24)
  override def firstTime: RichDate = RichDate("2020-03-26")

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    val maxSamplesPerClass: Int = args.int("samples_per_query_tweet_class", default = 2000)
    val timeframe: Int = 30
    val outputPath: String =
      s"/user/cassowary/processed/tweet_similarity/unhydrated_pairs_${timeframe}min"

    UnhydratedPairsCollectionJob.getLabelledPairs(
      dateRange,
      timeframe.minute.inMilliseconds,
      maxSamplesPerClass,
      TweetSimilarityUnhydratedPairs30MinScalaDataset,
      outputPath)
  }
}

object UnhydratedPairsCollection120MinScheduledApp extends ScheduledExecutionApp {

  override def batchIncrement: Duration = Hours(24)
  override def firstTime: RichDate = RichDate("2020-03-26")

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    val maxSamplesPerClass: Int = args.int("samples_per_query_tweet_class", default = 2000)
    val timeframe: Int = 120
    val outputPath: String =
      s"/user/cassowary/processed/tweet_similarity/unhydrated_pairs_${timeframe}min"

    UnhydratedPairsCollectionJob.getLabelledPairs(
      dateRange,
      timeframe.minute.inMilliseconds,
      maxSamplesPerClass,
      TweetSimilarityUnhydratedPairs120MinScalaDataset,
      outputPath)
  }
}
