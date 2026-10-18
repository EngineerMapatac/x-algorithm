package com.twitter.hub.pmi.math.aggregators

import com.twitter.scalding.TypedPipe
import com.twitter.algebird.QTree
import com.twitter.algebird.QTreeSemigroup
import com.twitter.algebird.Semigroup

object Percentiles {

  def getPresentPercentiles[T](
    pmiSet: TypedPipe[(T, Double)],
    percentiles: Seq[Double],
    offset: Int,
    k: Int
  )(
    implicit ordering: Ordering[T]
  ): TypedPipe[(T, Map[Double, Double])] = {
    implicit val qtSemigroup: Semigroup[QTree[Double]] = new QTreeSemigroup[Double](k)

    pmiSet
      .map(pmiSet => (pmiSet._1, QTree(pmiSet._2 + offset)))
      .sumByKey
      .mapValues(qTree =>
        percentiles.map { percentile =>
          val bounds = qTree.quantileBounds(percentile)
          val mean = (bounds._1 + bounds._2) / 2.0 - offset
          (percentile, mean)
        }.toMap)
  }
}
