package com.twitter.simclusters_v2.summingbird.common

import com.twitter.algebird.Monoid
import com.twitter.summingbird._

object SummerWithSumValues {
  implicit class SummerWithSumValues[P <: Platform[P], K, V](
    summer: Summer[P, K, V]) {
    def sumValues(monoid: Monoid[V]): KeyedProducer[P, K, V] =
      summer.mapValues {
        case (Some(oldV), deltaV) => monoid.plus(oldV, deltaV)
        case (None, deltaV) => deltaV
      }
  }
}
