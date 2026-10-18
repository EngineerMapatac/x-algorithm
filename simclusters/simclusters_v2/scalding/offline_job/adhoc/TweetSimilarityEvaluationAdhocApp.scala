package com.twitter.simclusters_v2.scalding.offline_job.adhoc

import com.twitter.bijection.Bufferable
import com.twitter.bijection.Injection
import com.twitter.scalding._
import com.twitter.scalding.commons.source.VersionedKeyValSource
import com.twitter.simclusters_v2.common.ClusterId
import com.twitter.simclusters_v2.common.CosineSimilarityUtil
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.scalding.common.matrix.SparseRowMatrix
import com.twitter.simclusters_v2.scalding.offline_job.SimClustersOfflineJobUtil
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import java.util.TimeZone

object TweetSimilarityEvaluationSamplingAdhocApp extends AdhocExecutionApp {

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {

    val random = new java.util.Random(args.long("seed", 20200322L))

    val topK = args.int("bucket_size", 1000)

    val output = args("output")

    SimClustersOfflineJobUtil
      .readTimelineFavoriteData(dateRange)
      .map {
        case (_, tweetId, _) =>
          tweetId -> 1L
      }
      .sumByKey
      .filter(_._2 >= 10L)
      .map {
        case (tweetId, tweetFavs) =>
          val bucket = math.log10(tweetFavs + 1.0).toInt
          bucket -> (tweetId, random.nextDouble())
      }
      .group
      .sortedReverseTake(topK)(Ordering.by(_._2))
      .flatMap {
        case (bucket, tweets) =>
          val bucketSize = tweets.length
          tweets.map {
            case (tweetId, _) =>
              (tweetId, bucket, bucketSize)
          }
      }
      .writeExecution(
        TypedTsv[(Long, Int, Int)](output)
      )

  }
}

object TweetSimilarityEvaluationAdhocApp extends AdhocExecutionApp {

  implicit val inj1: Injection[List[(Int, Double)], Array[Byte]] =
    Bufferable.injectionOf[List[(Int, Double)]]
  implicit val inj2: Injection[List[(Long, Double)], Array[Byte]] =
    Bufferable.injectionOf[List[(Long, Double)]]

  private def formatList(candidates: Seq[(TweetId, Double)]): Seq[(TweetId, Int)] = {
    candidates.take(10).map {
      case (clusterId, score) =>
        (clusterId, (score * 100).toInt)
    }
  }

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {

    val tweetTopKClustersPath = args("tweet_top_k")

    val clusterTopKTweetsPath = args("cluster_top_k")

    val tweetsPath = args("tweets")

    val threshold = args.double("threshold", 0.8)

    val topK = args.int("topK", 100)

    val output = args("output")

    val tweetTopKClusters: SparseRowMatrix[TweetId, ClusterId, Double] =
      SparseRowMatrix(
        TypedPipe
          .from(
            VersionedKeyValSource[TweetId, List[(ClusterId, Double)]](tweetTopKClustersPath)
          )
          .mapValues(_.filter(_._2 > 0.001).toMap),
        isSkinnyMatrix = true
      ).rowL2Normalize

    val clusterTopTweets: SparseRowMatrix[ClusterId, TweetId, Double] =
      SparseRowMatrix(
        TypedPipe
          .from(
            VersionedKeyValSource[ClusterId, List[(TweetId, Double)]](clusterTopKTweetsPath)
          )
          .mapValues(_.filter(_._2 > 0.02).toMap),
        isSkinnyMatrix = false
      )

    val tweetSubset = TypedPipe.from(TypedTsv[(Long, Int, Int)](tweetsPath))

    val tweetEmbeddingSubset =
      tweetTopKClusters.filterRows(tweetSubset.map(_._1))

    val groundTruthData = tweetTopKClusters.toSparseMatrix
      .multiplySkinnySparseRowMatrix(
        tweetEmbeddingSubset.toSparseMatrix.transpose.toSparseRowMatrix(true),
        numReducersOpt = Some(5000)
      )
      .toSparseMatrix
      .transpose
      .filter((_, _, v) => v > threshold)
      .sortWithTakePerRow(topK)(Ordering.by(-_._2))

    val predictionData = clusterTopTweets.toSparseMatrix.transpose
      .multiplySkinnySparseRowMatrix(
        tweetEmbeddingSubset.toSparseMatrix.transpose.toSparseRowMatrix(true),
        numReducersOpt = Some(5000)
      )
      .toSparseMatrix
      .transpose
      .toTypedPipe
      .map {
        case (queryTweet, candidateTweet, _) =>
          (queryTweet, candidateTweet)
      }
      .join(tweetEmbeddingSubset.toTypedPipe)
      .map {
        case (queryId, (candidateId, queryEmbedding)) =>
          candidateId -> (queryId, queryEmbedding)
      }
      .join(tweetTopKClusters.toTypedPipe)
      .map {
        case (candidateId, ((queryId, queryEmbedding), candidateEmbedding)) =>
          queryId -> (candidateId, CosineSimilarityUtil
            .dotProduct(
              queryEmbedding,
              candidateEmbedding
            ))
      }
      .filter(_._2._2 > threshold)
      .group
      .sortedReverseTake(topK)(Ordering.by(_._2))

    val potentialData =
      groundTruthData
        .leftJoin(predictionData)
        .map {
          case (tweetId, (groundTruthCandidates, predictedCandidates)) =>
            val predictedCandidateSet = predictedCandidates.toSeq.flatten.map(_._1).toSet
            val potentialTweets = groundTruthCandidates.filterNot {
              case (candidateId, _) =>
                predictedCandidateSet.contains(candidateId)
            }
            (tweetId, potentialTweets)
        }

    val debuggingData =
      groundTruthData
        .leftJoin(predictionData)
        .map {
          case (tweetId, (groundTruthTweets, maybepredictedTweets)) =>
            val predictedTweets = maybepredictedTweets.toSeq.flatten
            val predictedTweetSet = predictedTweets.map(_._1).toSet
            val potentialTweets = groundTruthTweets.filterNot {
              case (candidateId, _) =>
                predictedTweetSet.contains(candidateId)
            }

            (
              tweetId,
              Seq(
                formatList(potentialTweets),
                formatList(groundTruthTweets),
                formatList(predictedTweets)))
        }

    val eval = tweetSubset
      .map {
        case (tweetId, bucket, bucketSize) =>
          tweetId -> (bucket, bucketSize)
      }
      .leftJoin(groundTruthData)
      .leftJoin(predictionData)
      .map {
        case (_, (((bucket, bucketSize), groundTruthOpt), predictionOpt)) =>
          val groundTruth = groundTruthOpt.getOrElse(Nil).map(_._1)
          val prediction = predictionOpt.getOrElse(Nil).map(_._1)

          assert(groundTruth.distinct.size == groundTruth.size)
          assert(prediction.distinct.size == prediction.size)

          val intersection = groundTruth.toSet.intersect(prediction.toSet)

          val precision =
            if (prediction.nonEmpty)
              intersection.size.toDouble / prediction.size.toDouble
            else 0.0
          val recall =
            if (groundTruth.nonEmpty)
              intersection.size.toDouble / groundTruth.size.toDouble
            else 0.0

          (
            bucket,
            bucketSize) -> (groundTruth.size, prediction.size, intersection.size, precision, recall, 1.0)
      }
      .sumByKey
      .map {
        case (
              (bucket, bucketSize),
              (groundTruthSum, predictionSum, interSectionSum, precisionSum, recallSum, count)) =>
          (
            bucket,
            bucketSize,
            groundTruthSum / count,
            predictionSum / count,
            interSectionSum / count,
            precisionSum / count,
            recallSum / count,
            count)
      }

    Execution
      .zip(
        eval
          .writeExecution(TypedTsv(output)),
        groundTruthData
          .map {
            case (tweetId, neighbors) =>
              tweetId -> neighbors
                .map {
                  case (id, score) => s"$id:$score"
                }
                .mkString(",")
          }
          .writeExecution(
            TypedTsv(args("output") + "_ground_truth")
          ),
        predictionData
          .map {
            case (tweetId, neighbors) =>
              tweetId -> neighbors
                .map {
                  case (id, score) => s"$id:$score"
                }
                .mkString(",")
          }
          .writeExecution(
            TypedTsv(args("output") + "_prediction")
          ),
        potentialData
          .map {
            case (tweetId, neighbors) =>
              tweetId -> neighbors
                .map {
                  case (id, score) => s"$id:$score"
                }
                .mkString(",")
          }.writeExecution(
            TypedTsv(args("output") + "_potential")
          ),
        debuggingData
          .map {
            case (tweetId, candidateList) =>
              val value = candidateList
                .map { candidates =>
                  candidates
                    .map {
                      case (id, score) =>
                        s"${id}D$score"
                    }.mkString("C")
                }.mkString("B")
              s"${tweetId}A$value"
          }.writeExecution(
            TypedTsv(args("output") + "_debugging")
          )
      )
      .unit
  }
}
