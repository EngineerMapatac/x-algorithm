package com.twitter.simclusters_v2.scalding.offline_job

import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.simclusters_v2.hdfs_sources._
import com.twitter.simclusters_v2.scalding.offline_job.SimClustersOfflineJob._
import com.twitter.simclusters_v2.scalding.offline_job.SimClustersOfflineJobUtil._
import com.twitter.simclusters_v2.thriftscala.TweetAndClusterScores
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import java.util.TimeZone

object SimClustersOfflineJobScheduledApp extends ScheduledExecutionApp {
  import com.twitter.simclusters_v2.scalding.common.TypedRichPipe._

  private val tweetClusterScoresDatasetPath: String =
    "/user/cassowary/processed/simclusters/tweet_cluster_scores"
  private val tweetTopKClustersDatasetPath: String =
    "/user/cassowary/processed/simclusters/tweet_top_k_clusters"
  private val clusterTopKTweetsDatasetPath: String =
    "/user/cassowary/processed/simclusters/cluster_top_k_tweets"

  override def batchIncrement: Duration = Hours(12)

  override def firstTime: RichDate = RichDate("2025-02-24")

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {

    val previousTweetClusterScores: TypedPipe[TweetAndClusterScores] =
      if (firstTime.timestamp == dateRange.start.timestamp) {
        TypedPipe.from(Nil)
      } else {
        DAL
          .readMostRecentSnapshot(
            SimclustersOfflineTweetClusterScoresScalaDataset,
            dateRange - batchIncrement
          )
          .toTypedPipe
          .count("NumPreviousTweetClusterScores")
      }

    val tweetsToKeep = getSubsetOfValidTweets(Days(1))
      .count("NumTweetsToKeep")

    val updatedTweetClusterScores = computeAggregatedTweetClusterScores(
      dateRange,
      readInterestedInScalaDataset(dateRange),
      readTimelineFavoriteData(dateRange),
      previousTweetClusterScores
    ).map { tweetClusterScore =>
        tweetClusterScore.tweetId -> tweetClusterScore
      }
      .count("NumUpdatedTweetClusterScoresBeforeFiltering")
      .join(tweetsToKeep.asKeys)
      .map {
        case (_, (tweetClusterScore, _)) => tweetClusterScore
      }
      .count("NumUpdatedTweetClusterScores")
      .forceToDisk

    val tweetTopKClusters = computeTweetTopKClusters(updatedTweetClusterScores)
      .count("NumTweetTopKSaved")
    val clusterTopKTweets = computeClusterTopKTweets(updatedTweetClusterScores)
      .count("NumClusterTopKSaved")

    val writeTweetClusterScoresExec = updatedTweetClusterScores
      .writeDALSnapshotExecution(
        SimclustersOfflineTweetClusterScoresScalaDataset,
        D.Hourly,
        D.Suffix(tweetClusterScoresDatasetPath),
        D.Parquet,
        dateRange.end
      )

    val writeTweetTopKClustersExec = tweetTopKClusters
      .writeDALSnapshotExecution(
        SimclustersOfflineTweetTopKClustersScalaDataset,
        D.Hourly,
        D.Suffix(tweetTopKClustersDatasetPath),
        D.Parquet,
        dateRange.end
      )

    val writeClusterTopKTweetsExec = clusterTopKTweets
      .writeDALSnapshotExecution(
        SimclustersOfflineClusterTopKTweetsScalaDataset,
        D.Hourly,
        D.Suffix(clusterTopKTweetsDatasetPath),
        D.Parquet,
        dateRange.end
      )

    Execution
      .zip(writeTweetClusterScoresExec, writeTweetTopKClustersExec, writeClusterTopKTweetsExec)
      .unit
  }

}
