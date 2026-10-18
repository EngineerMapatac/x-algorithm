package com.twitter.simclusters_v2.scalding.offline_job.adhoc

import com.twitter.bijection.Bufferable
import com.twitter.bijection.Injection
import com.twitter.scalding._
import com.twitter.scalding.commons.source.VersionedKeyValSource
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.common.ClusterId
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.hdfs_sources.SimclustersV2InterestedIn20M145KUpdatedScalaDataset
import com.twitter.simclusters_v2.scalding.common.matrix.SparseMatrix
import com.twitter.simclusters_v2.scalding.common.matrix.SparseRowMatrix
import com.twitter.simclusters_v2.scalding.offline_job.SimClustersOfflineJobUtil
import com.twitter.simclusters_v2.summingbird.common.Configs
import com.twitter.simclusters_v2.summingbird.common.SimClustersInterestedInUtil
import com.twitter.simclusters_v2.thriftscala.ClustersUserIsInterestedIn
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import java.util.TimeZone

object SimClustersTweetEmbeddingAdhocApp extends AdhocExecutionApp {

  import SimClustersOfflineJobUtil._

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {

    val outputDir = args("output_dir")

    val scoringMethod = args.getOrElse("score_type", "logFav")

    val usingNormalizedScoringFunction = args.boolean("run_normalized")

    val tweetFavThreshold = args.long("tweet_fav_threshold", 0L)

    val tweetTopKClustersOutputPath: String = outputDir + "/tweet_top_k_clusters"

    val clusterTopKTweetsOutputPath: String = outputDir + "/cluster_top_k_tweets"

    val interestedInData: TypedPipe[(Long, ClustersUserIsInterestedIn)] =
      DAL
        .readMostRecentSnapshot(
          SimclustersV2InterestedIn20M145KUpdatedScalaDataset,
          dateRange.embiggen(Days(14))
        )
        .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
        .toTypedPipe
        .map {
          case KeyVal(key, value) => (key, value)
        }

    val userTweetFavData: SparseMatrix[UserId, TweetId, Double] =
      SparseMatrix(readTimelineFavoriteData(dateRange)).tripleApply {
        case (userId, tweetId, timestamp) =>
          (
            userId,
            tweetId,
            thriftDecayedValueMonoid
              .plus(
                thriftDecayedValueMonoid.build(1.0, timestamp),
                thriftDecayedValueMonoid.build(0.0, dateRange.end.timestamp)
              )
              .value)
      }

    val tweetSubset =
      userTweetFavData.colNnz.filter(
        _._2 > tweetFavThreshold.toDouble
      )

    val userTweetFavDataSubset = userTweetFavData.filterCols(tweetSubset.keys)

    val userSimClustersInterestedInData: SparseRowMatrix[UserId, ClusterId, Double] =
      SparseRowMatrix(
        interestedInData.map {
          case (userId, clusters) =>
            val topClustersWithScores =
              SimClustersInterestedInUtil
                .topClustersWithScores(clusters)
                .collect {
                  case (clusterId, scores)
                      if scores.favScore > Configs
                        .favScoreThresholdForUserInterest(
                          clusters.knownForModelVersion
                        ) =>
                    scoringMethod match {
                      case "fav" =>
                        clusterId -> scores.clusterNormalizedFavScore
                      case "follow" =>
                        clusterId -> scores.clusterNormalizedFollowScore
                      case "logFav" =>
                        clusterId -> scores.clusterNormalizedLogFavScore
                      case _ =>
                        throw new IllegalArgumentException(
                          "score_type can only be fav, follow or logFav")
                    }
                }
                .filter(_._2 > 0.0)
                .toMap
            userId -> topClustersWithScores
        },
        isSkinnyMatrix = true
      )

    val tweetClusterScoreMatrix = if (usingNormalizedScoringFunction) {
      userTweetFavDataSubset.transpose.rowL2Normalize
        .multiplySkinnySparseRowMatrix(userSimClustersInterestedInData)
    } else {
      userTweetFavDataSubset.transpose.multiplySkinnySparseRowMatrix(
        userSimClustersInterestedInData)
    }

    val tweetTopClusters = tweetClusterScoreMatrix
      .sortWithTakePerRow(Configs.topKClustersPerTweet)(Ordering.by(-_._2))
      .fork

    val clusterTopTweets = tweetClusterScoreMatrix
      .sortWithTakePerCol(Configs.topKTweetsPerCluster)(Ordering.by(-_._2))
      .fork

    implicit val inj1: Injection[List[(Int, Double)], Array[Byte]] =
      Bufferable.injectionOf[List[(Int, Double)]]
    implicit val inj2: Injection[List[(Long, Double)], Array[Byte]] =
      Bufferable.injectionOf[List[(Long, Double)]]

    Execution
      .zip(
        tweetTopClusters
          .mapValues(_.toList)
          .writeExecution(
            VersionedKeyValSource[TweetId, List[(ClusterId, Double)]](tweetTopKClustersOutputPath)
          ),
        tweetTopClusters
          .map {
            case (tweetId, topKClusters) =>
              tweetId -> topKClusters
                .map {
                  case (clusterId, score) =>
                    s"$clusterId:" + "%.3g".format(score)
                }
                .mkString(",")
          }
          .writeExecution(
            TypedTsv(tweetTopKClustersOutputPath + "_tsv")
          ),
        tweetSubset.writeExecution(TypedTsv(tweetTopKClustersOutputPath + "_tweet_favs")),
        clusterTopTweets
          .mapValues(_.toList)
          .writeExecution(
            VersionedKeyValSource[ClusterId, List[(TweetId, Double)]](clusterTopKTweetsOutputPath)
          ),
        clusterTopTweets
          .map {
            case (clusterId, topKTweets) =>
              clusterId -> topKTweets
                .map {
                  case (tweetId, score) => s"$tweetId:" + "%.3g".format(score)
                }
                .mkString(",")
          }
          .writeExecution(
            TypedTsv(clusterTopKTweetsOutputPath + "_tsv")
          )
      )
      .unit
  }
}
