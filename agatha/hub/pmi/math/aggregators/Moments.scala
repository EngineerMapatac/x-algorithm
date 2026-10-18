package com.twitter.hub.pmi.math.aggregators

import com.twitter.hub.pmi.PMISetExtractors.EntityCount
import com.twitter.scalding.TypedPipe
import com.twitter.algebird.MultiAggregator
import com.twitter.algebird.Aggregator.prepareMonoid
import com.twitter.algebird.Aggregator.size

object Moments {
  class Moments(private val moments: Map[Int, Double], maxMoment: Int) {
    val count: Double = moments(0)
    val mean: Double = moments(1)
    val m0: Double = mean

    lazy val toSparseMap: Map[Int, Double] = moments
    lazy val toDenseMap: Map[Int, Double] = (moments.keys.max + 1)
      .to(maxMoment)
      .map(i => (i, Double.NaN))
      .toMap ++ moments

    lazy val toSparseSeq: Seq[(Int, Double)] = moments.toSeq
    lazy val toDenseSeq: Seq[(Int, Double)] = toDenseMap.toSeq
    lazy val sparseSize: Int = moments.size
    lazy val denseSize: Int = toDenseMap.size

    def sparseContains(k: Int): Boolean = moments.contains(k)

    def denseContains(k: Int): Boolean = toDenseMap.contains(k)

    def get(i: Int): Option[Double] = if (i > maxMoment)
      None
    else
      Some(moments.getOrElse(i, Double.NaN))

    def apply(i: Int): Double = if (i > maxMoment)
      moments(i)
    else
      moments.getOrElse(i, Double.NaN)

  }

  case class StandardizedMoments(
    private val moments: Map[Int, Double],
    maxMoment: Int)
      extends Moments(moments, maxMoment)

  case class CentralMoments(
    private val moments: Map[Int, Double],
    maxMoment: Int)
      extends Moments(moments, maxMoment) {
    lazy val standardize: StandardizedMoments = {
      val newMap =
        if (maxMoment <= 2 || apply(2).isNaN)
          moments
        else {
          val sd = math.sqrt(moments(2))
          moments.map {
            case (idx, value) =>
              if (idx <= 2) (idx, value)
              else (idx, value / math.pow(sd, idx))
          }
        }
      StandardizedMoments(newMap, maxMoment)
    }
  }

  def getMoments[T, U](
    pmiSet: TypedPipe[((T, U), Double)],
    entityCounts: TypedPipe[EntityCount[T]],
    nMoments: Int
  )(
    implicit orderingT: Ordering[T],
    orderingU: Ordering[U]
  ): TypedPipe[((T, U), CentralMoments)] = {

    val sum = prepareMonoid[Double, Double](x => x)
    val multiAgg = MultiAggregator((size, sum))

    val pmiStats = pmiSet.group
      .aggregate(multiAgg)
      .groupBy { case ((entityId, _), _) => entityId }
      .join(entityCounts.groupBy(_.entityId))
      .mapValues {
        case ((entityLabelSet, (pmiSize, pmiSum)), entityCount) =>
          (
            entityLabelSet,
            (
              entityCount.count.toDouble - pmiSize.toDouble,
              entityCount.count.toDouble,
              pmiSum / entityCount.count.toDouble
            )
          )

      }
      .values
      .forceToDisk

    val firstMoments = pmiStats
      .mapValues {
        case (zeroSize, totalSize, mean) =>
          Map(
            -1 -> zeroSize,
            0 -> totalSize,
            1 -> mean
          )
      }

    val additionalMoments = pmiSet
      .join(pmiStats)
      .flatMapValues {
        case (pmi, (_, totalSize, mean)) =>
          (2 to math.min(nMoments, totalSize.toInt)).map(i => (i, math.pow(pmi - mean, i)))
      }
      .map {
        case (entityLabelSet, (momentIdx, centeredPmi)) =>
          ((entityLabelSet, momentIdx), centeredPmi)
      }
      .sumByKey
      .map {
        case ((entityLabelSet, momentIdx), centeredPmiSum) =>
          (entityLabelSet, (momentIdx, centeredPmiSum))
      }
      .join(pmiStats)
      .mapValues {
        case ((momentIdx, centeredPmiSum), (zeroSize, totalSize, mean)) =>
          val extraZeros = zeroSize * math.pow(-mean, momentIdx)
          val momentValue = (centeredPmiSum + extraZeros) / totalSize
          Map(momentIdx -> momentValue)
      }
      .reduce(_ ++ _)

    firstMoments
      .leftJoin(additionalMoments)
      .mapValues {
        case (first, additional) =>
          CentralMoments(first ++ additional.getOrElse(Map.empty), nMoments)
      }
  }

  def getPresentMoments[T](
    pmiSet: TypedPipe[(T, Double)]
  )(
    implicit ordering: Ordering[T]
  ): TypedPipe[(T, CentralMoments)] = {
    pmiSet.group
      .aggregate(com.twitter.algebird.Moments.aggregator)
      .mapValues { moments =>
        val count = moments.m0.toDouble
        val momentMap = Map(0 -> count, 1 -> moments.m1) ++ {
          if (count >= 2) Map(2 -> moments.m2 / count) else Map.empty
        } ++ { if (count >= 3) Map(3 -> moments.m3 / count) else Map.empty } ++ {
          if (count >= 4) Map(4 -> moments.m4 / count) else Map.empty
        }
        CentralMoments(
          momentMap,
          4
        )
      }
  }
}
