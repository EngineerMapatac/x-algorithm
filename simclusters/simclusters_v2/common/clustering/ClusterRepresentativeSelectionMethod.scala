package com.twitter.simclusters_v2.common.clustering

import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.thriftscala.NeighborWithWeights

trait ClusterRepresentativeSelectionMethod[T] {

  def selectClusterRepresentative(
    cluster: Set[NeighborWithWeights],
    embeddings: Map[UserId, T]
  ): UserId

}

object ClusterRepresentativeSelectionStatistics {

  val StatClusterRepresentativeSelectionTime = "cluster_representative_selection_total_time_ms"
}
