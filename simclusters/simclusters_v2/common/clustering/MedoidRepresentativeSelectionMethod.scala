package com.twitter.simclusters_v2.common.clustering

import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.thriftscala.NeighborWithWeights

class MedoidRepresentativeSelectionMethod[T](
  producerProducerSimilarityFn: (T, T) => Double)
    extends ClusterRepresentativeSelectionMethod[T] {

  def selectClusterRepresentative(
    cluster: Set[NeighborWithWeights],
    embeddings: Map[UserId, T],
  ): UserId = {
    val key = cluster.maxBy { id1 =>
      val v = embeddings(id1.neighborId)
      cluster
        .map(id2 => producerProducerSimilarityFn(v, embeddings(id2.neighborId))).sum
    }
    key.neighborId
  }
}
