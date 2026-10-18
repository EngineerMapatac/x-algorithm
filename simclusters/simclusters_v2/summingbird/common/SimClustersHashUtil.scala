package com.twitter.simclusters_v2.summingbird.common

object SimClustersHashUtil {
  def clusterIdToBucket(clusterId: Int): Int = {
    clusterId % numBuckets
  }

  val numBuckets: Int = 200

  val getAllBuckets: Seq[Int] = 0.until(numBuckets)
}
