package com.twitter.simclusters_v2.common.clustering

import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.thriftscala.NeighborWithWeights

class MaxFavScoreRepresentativeSelectionMethod[T] extends ClusterRepresentativeSelectionMethod[T] {

  def selectClusterRepresentative(
    cluster: Set[NeighborWithWeights],
    embeddings: Map[UserId, T],
  ): UserId = {
    val key = cluster.maxBy { x: NeighborWithWeights => x.favScoreHalfLife100Days.getOrElse(0.0) }
    key.neighborId
  }
}
