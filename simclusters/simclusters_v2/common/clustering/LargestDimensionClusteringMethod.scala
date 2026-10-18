package com.twitter.simclusters_v2.common.clustering

class LargestDimensionClusteringMethod extends ClusteringMethod {

  override def cluster[T](
    embeddings: Map[Long, T],
    similarityFn: (T, T) => Double,
    recordStatCallback: (String, Long) => Unit
  ): Set[Set[Long]] = {

    new ConnectedComponentsClusteringMethod(similarityThreshold = 0.1)
      .cluster(embeddings, similarityFn, recordStatCallback)
  }

}
