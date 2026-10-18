package com.twitter.simclusters_v2.scalding.embedding.abuse

import com.twitter.simclusters_v2.common.ClusterId
import com.twitter.simclusters_v2.thriftscala.SimClusterWithScore
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbedding
import com.twitter.util.Try

object ClusterPair {
  def apply(
    clusterId: ClusterId,
    healthyScore: Double,
    unhealthyScore: Double
  ): Option[ClusterPair] = {
    if (healthyScore + unhealthyScore == 0.0) {
      None
    } else {
      Some(new ClusterPair(clusterId, healthyScore, unhealthyScore))
    }
  }
}

case class ClusterPair private (
  clusterId: ClusterId,
  healthyScore: Double,
  unhealthyScore: Double) {

  def totalScores: Double = healthyScore + unhealthyScore

  def healthRatio: Double = unhealthyScore / (unhealthyScore + healthyScore)
}

object PairedInteractionFeatures {
  def smoothedHealthRatio(
    unhealthySum: Double,
    healthySum: Double,
    smoothingFactor: Double,
    prior: Double
  ): Double =
    (unhealthySum + smoothingFactor * prior) / (unhealthySum + healthySum + smoothingFactor)
}

case class PairedInteractionFeatures(
  healthyInteractionSimClusterEmbedding: SimClustersEmbedding,
  unhealthyInteractionSimClusterEmbedding: SimClustersEmbedding) {

  private[this] val scorePairs: Seq[ClusterPair] = {
    val clusterToScoreMap = healthyInteractionSimClusterEmbedding.embedding.map {
      simClusterWithScore =>
        simClusterWithScore.clusterId -> simClusterWithScore.score
    }.toMap

    unhealthyInteractionSimClusterEmbedding.embedding.flatMap { simClusterWithScore =>
      val clusterId = simClusterWithScore.clusterId
      val postiveScoreOption = clusterToScoreMap.get(clusterId)
      postiveScoreOption.flatMap { postiveScore =>
        ClusterPair(clusterId, postiveScore, simClusterWithScore.score)
      }
    }
  }

  val highestScoreClusterPair: Option[ClusterPair] =
    Try(scorePairs.maxBy(_.totalScores)).toOption

  val highestHealthRatioClusterPair: Option[ClusterPair] =
    Try(scorePairs.maxBy(_.healthRatio)).toOption

  val lowestHealthRatioClusterPair: Option[ClusterPair] =
    Try(scorePairs.minBy(_.healthRatio)).toOption

  val healthRatioEmbedding: SimClustersEmbedding = {
    val scores = scorePairs.map { pair =>
      SimClusterWithScore(pair.clusterId, pair.healthRatio)
    }
    SimClustersEmbedding(scores)
  }

  val healthySum: Double = healthyInteractionSimClusterEmbedding.embedding.map(_.score).sum

  val unhealthySum: Double = unhealthyInteractionSimClusterEmbedding.embedding.map(_.score).sum

  val healthRatio: Double = unhealthySum / (unhealthySum + healthySum)

  def smoothedHealthRatio(smoothingFactor: Double, prior: Double): Double =
    PairedInteractionFeatures.smoothedHealthRatio(unhealthySum, healthySum, smoothingFactor, prior)
}
