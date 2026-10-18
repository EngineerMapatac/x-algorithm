package com.twitter.simclusters_v2.common.clustering

trait ClusteringMethod {

  def cluster[T](
    embeddings: Map[Long, T],
    similarityFn: (T, T) => Double,
    recordStatCallback: (String, Long) => Unit = (_, _) => ()
  ): Set[Set[Long]]

}

object ClusteringStatistics {

  val StatSimilarityGraphTotalBuildTime = "similarity_graph_total_build_time_ms"
  val StatClusteringAlgorithmRunTime = "clustering_algorithm_total_run_time_ms"
  val StatMedoidSelectionTime = "medoid_selection_total_time_ms"
  val StatComputedSimilarityBeforeFilter = "computed_similarity_before_filter"

}
