package com.twitter.simclustersann.candidate_source

import com.twitter.simclusters_v2.common.ClusterId
import com.twitter.simclusters_v2.common.SimClustersEmbedding
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.thriftscala.InternalId
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingId
import com.twitter.simclustersann.thriftscala.ScoringAlgorithm
import com.twitter.simclustersann.thriftscala.SimClustersANNConfig
import com.twitter.snowflake.id.SnowflakeId
import com.twitter.util.Duration
import com.twitter.util.Time
import scala.collection.mutable
import com.twitter.simclusters_v2.thriftscala.SimplifiedClusterDetails

trait ApproximateCosineSimilarity {
  type ScoredTweet = (Long, Double)
  def apply(
    sourceEmbedding: SimClustersEmbedding,
    sourceEmbeddingId: SimClustersEmbeddingId,
    config: SimClustersANNConfig,
    candidateScoresStat: Int => Unit,
    clusterTweetsMap: Map[ClusterId, Option[Seq[(TweetId, Double, Double)]]],
    clusterTweetsMapArray: Map[ClusterId, Option[Array[(TweetId, Double)]]] = Map.empty,
    clusterDetailsMap: Map[String, Option[SimplifiedClusterDetails]]
  ): Seq[ScoredTweet]
}

object ApproximateCosineSimilarity extends ApproximateCosineSimilarity {

  final val InitialCandidateMapSize = 16384
  val MaxNumResultsUpperBound = 1000
  final val MaxTweetCandidateAgeUpperBound = 175200

  private class HashMap[A, B](initSize: Int) extends mutable.HashMap[A, B] {
    override def initialSize: Int = initSize
  }

  private def parseTweetId(embeddingId: SimClustersEmbeddingId): Option[TweetId] = {
    embeddingId.internalId match {
      case InternalId.TweetId(tweetId) =>
        Some(tweetId)
      case _ =>
        None
    }
  }

  override def apply(
    sourceEmbedding: SimClustersEmbedding,
    sourceEmbeddingId: SimClustersEmbeddingId,
    config: SimClustersANNConfig,
    candidateScoresStat: Int => Unit,
    clusterTweetsMap: Map[ClusterId, Option[Seq[(TweetId, Double, Double)]]],
    clusterTweetsMapArray: Map[ClusterId, Option[Array[(TweetId, Double)]]],
    clusterDetailsMap: Map[String, Option[SimplifiedClusterDetails]]
  ): Seq[ScoredTweet] = {
    val now = Time.now
    val earliestTweetId =
      if (config.maxTweetCandidateAgeHours >= MaxTweetCandidateAgeUpperBound)
        0L
      else
        SnowflakeId.firstIdFor(now - Duration.fromHours(config.maxTweetCandidateAgeHours))
    val latestTweetId =
      SnowflakeId.firstIdFor(now - Duration.fromHours(config.minTweetCandidateAgeHours))

    val candidateScoresMap =
      new HashMap[TweetId, Double](InitialCandidateMapSize)
    val candidateNormalizationMap =
      new HashMap[TweetId, Double](InitialCandidateMapSize)
    val candidateFavSquaredL2NormMap =
      new HashMap[TweetId, Double](InitialCandidateMapSize)

    clusterTweetsMap.foreach {
      case (clusterId, Some(tweetScores)) if sourceEmbedding.contains(clusterId) =>
        val sourceClusterScore = sourceEmbedding.getOrElse(clusterId)

        for (i <- 0 until Math.min(tweetScores.size, config.maxTopTweetsPerCluster)) {
          val (tweetId, score, favSquaredL2Norm) = tweetScores(i)

          if (!parseTweetId(sourceEmbeddingId).contains(tweetId) &&
            tweetId >= earliestTweetId && tweetId <= latestTweetId) {
            candidateScoresMap.put(
              tweetId,
              candidateScoresMap.getOrElse(tweetId, 0.0) + score * sourceClusterScore)
            candidateNormalizationMap
              .put(tweetId, candidateNormalizationMap.getOrElse(tweetId, 0.0) + score * score)
            candidateFavSquaredL2NormMap.put(tweetId, favSquaredL2Norm)
          }
        }
      case _ => ()
    }

    candidateScoresStat(candidateScoresMap.size)

    val processedCandidateScores: Seq[(TweetId, Double)] = candidateScoresMap.map {
      case (candidateId, score) =>
        val processedScore = {
          config.annAlgorithm match {
            case ScoringAlgorithm.LogCosineSimilarity =>
              score / sourceEmbedding.logNorm / math.log(1 + candidateNormalizationMap(candidateId))
            case ScoringAlgorithm.CosineSimilarity =>
              score / sourceEmbedding.l2norm / math.sqrt(candidateNormalizationMap(candidateId))
            case ScoringAlgorithm.CosineSimilarityFavL2NormBased =>
              score / sourceEmbedding.l2norm / math.sqrt(
                math.max(
                  candidateNormalizationMap(candidateId),
                  candidateFavSquaredL2NormMap(candidateId)))
            case ScoringAlgorithm.CosineSimilarityFavL2NormBasedWithExploration =>
              val cosineScore =
                score / sourceEmbedding.l2norm / math.sqrt(candidateNormalizationMap(candidateId))
              val favL2NormScore = score / sourceEmbedding.l2norm / math.sqrt(
                math.max(
                  candidateNormalizationMap(candidateId),
                  candidateFavSquaredL2NormMap(candidateId)))
              if (cosineScore > config.minScore && favL2NormScore > cosineScore / math.sqrt(2)) {
                cosineScore
              } else {
                favL2NormScore
              }
            case ScoringAlgorithm.CosineSimilarityNoSourceEmbeddingNormalization =>
              score / math.sqrt(candidateNormalizationMap(candidateId))
            case ScoringAlgorithm.DotProduct => score
          }
        }
        candidateId -> processedScore
    }.toSeq

    processedCandidateScores
      .filter(_._2 >= config.minScore)
      .sortBy(-_._2)
      .take(Math.min(config.maxNumResults, MaxNumResultsUpperBound))
  }
}
