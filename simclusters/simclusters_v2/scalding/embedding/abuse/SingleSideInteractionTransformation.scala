package com.twitter.simclusters_v2.scalding.embedding.abuse

import com.google.common.annotations.VisibleForTesting
import com.twitter.scalding._
import com.twitter.simclusters_v2.scalding.common.matrix.SparseMatrix
import com.twitter.simclusters_v2.scalding.embedding.common.EmbeddingUtil.ClusterId
import com.twitter.simclusters_v2.scalding.embedding.common.EmbeddingUtil.UserId
import com.twitter.simclusters_v2.thriftscala.AdhocSingleSideClusterScores
import com.twitter.simclusters_v2.thriftscala.SimClusterWithScore
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbedding

object SingleSideInteractionTransformation {

  def computeClusterFeatures(
    normalizedUserSimClusters: SparseMatrix[UserId, ClusterId, Double],
    interactionGraph: SparseMatrix[UserId, _, Double]
  ): TypedPipe[SimClusterWithScore] = {

    val numReportsForUserEntries = interactionGraph.rowL1Norms.map {
      case (user, count) => (user, 1, count)
    }

    val numReportsForUser = SparseMatrix[UserId, Int, Double](numReportsForUserEntries)

    normalizedUserSimClusters.transpose
      .multiplySparseMatrix(numReportsForUser)
      .toTypedPipe
      .map {
        case (clusterId, _, clusterScore: Double) =>
          SimClusterWithScore(clusterId, clusterScore)
      }
  }

  @VisibleForTesting
  private[abuse] def computeUserFeaturesFromClusters(
    normalizedUserSimClusters: SparseMatrix[UserId, ClusterId, Double],
    simClusterFeatures: TypedPipe[SimClusterWithScore]
  ): TypedPipe[(UserId, SimClustersEmbedding)] = {

    normalizedUserSimClusters.toTypedPipe
      .map {
        case (userId, clusterId, score) =>
          (clusterId, (userId, score))
      }
      .group
      .hashJoin(simClusterFeatures.groupBy(_.clusterId))
      .map {
        case (_, ((userId, score), singleSideClusterFeatures)) =>
          (
            userId,
            List(
              SimClusterWithScore(
                singleSideClusterFeatures.clusterId,
                singleSideClusterFeatures.score * score))
          )
      }
      .sumByKey
      .mapValues(SimClustersEmbedding.apply)
  }

  def pairScores(
    interactionMap: Map[String, TypedPipe[(UserId, SimClustersEmbedding)]]
  ): TypedPipe[AdhocSingleSideClusterScores] = {

    val combinedInteractions = interactionMap
      .map {
        case (interactionTypeName, userInteractionFeatures) =>
          userInteractionFeatures.map {
            case (userId, simClustersEmbedding) =>
              (userId, List((interactionTypeName, simClustersEmbedding)))
          }
      }
      .reduce[TypedPipe[(UserId, List[(String, SimClustersEmbedding)])]] {
        case (list1, list2) =>
          list1 ++ list2
      }
      .group
      .sumByKey

    combinedInteractions.toTypedPipe
      .map {
        case (userId, interactionFeatureList) =>
          AdhocSingleSideClusterScores(
            userId,
            interactionFeatureList.toMap
          )
      }
  }

  def clusterScoresFromGraphs(
    normalizedUserSimClusters: SparseMatrix[UserId, ClusterId, Double],
    interactionGraph: SparseMatrix[UserId, _, Double]
  ): TypedPipe[(UserId, SimClustersEmbedding)] = {
    val clusterFeatures = computeClusterFeatures(normalizedUserSimClusters, interactionGraph)
    computeUserFeaturesFromClusters(normalizedUserSimClusters, clusterFeatures)
  }
}
