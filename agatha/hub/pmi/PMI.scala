package com.twitter.hub.pmi

import com.twitter.algebird.Aggregator.head
import com.twitter.algebird.Aggregator.prepareMonoid
import com.twitter.algebird.Aggregator.size
import com.twitter.algebird.MultiAggregator
import com.twitter.hub.pmi.ModelCommon._
import com.twitter.hub.pmi.math.Math._
import com.twitter.hub.pmi.math.MathParams
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.ValuePipe
import scala.math.Ordered.orderingToOrdered

object PMI {

  case class RegularizationParams(
    minFeatureCount: Option[Int],
    maxFeatsPerInstance: Option[Int],
    maxFeatSumPerInstance: Option[Int],
  )

  case class PMIParams(
    data1Params: RegularizationParams,
    data2Params: RegularizationParams,
  )

  case class FeatureStats[T](featureId: T, featureValue: Double, featureSum: Double)

  case class PMIStats[T, U, V](
    instanceId: T,
    featureStats1: FeatureStats[U],
    featureStats2: FeatureStats[V],
    valueProd: Double)

  case class CrossedPMIStats[U, V](
    featureId1: U,
    featureId2: V,
    sumProd: Double,
    sumProdSq: Double,
    featureSum1: Double,
    featureSum2: Double,
    totalInstances: Long,
    pmi: Double)

  private def returnUnsmoothedMatrix[T, U, V](
    dataStore: PMITypes.PMIDataContainer[T, U, V],
    optFeatureCrossReducers: Option[Int]
  )(
    implicit orderingT: Ordering[T],
    orderingU: Ordering[U],
    serializerU: U => Array[Byte],
    orderingV: Ordering[V],
    serializerV: V => Array[Byte],
    jobParams: PMIParams,
    mathParams: MathParams,
    orderingUV: Ordering[(U, V)]
  ): TypedPipe[CrossedPMIStats[U, V]] = {
    val data1 = dataStore.data1
    val data2 = dataStore.data2
    val distinctInstanceCount: ValuePipe[Long] = (data1.map(_._1) ++ data2.map(_._1)).distinct
      .aggregate(size)

    val regularizedData1 = applyRegularizationToPipe(
      data1,
      jobParams.data1Params.maxFeatsPerInstance,
      jobParams.data1Params.maxFeatSumPerInstance
    ).forceToDisk

    val regularizedData2 = applyRegularizationToPipe(
      data2,
      jobParams.data2Params.maxFeatsPerInstance,
      jobParams.data2Params.maxFeatSumPerInstance
    ).forceToDisk

    val data1FeatureSums = getFeatureSumsWithMinFeatures(
      regularizedData1,
      jobParams.data1Params.minFeatureCount
    )

    val data2FeatureSums = getFeatureSumsWithMinFeatures(
      regularizedData2,
      jobParams.data2Params.minFeatureCount
    )

    val data1WithSums = dataStore
      .joinFeatureSumsToPipe1(
        regularizedData1,
        data1FeatureSums
      ).forceToDisk

    val data2WithSums = dataStore
      .joinFeatureSumsToPipe2(
        regularizedData2,
        data2FeatureSums
      ).forceToDisk

    val sumProd = prepareMonoid[PMIStats[T, U, V], Double](_.valueProd)
    val sumProdSq =
      prepareMonoid[PMIStats[T, U, V], Double](pmiStats => Math.pow(pmiStats.valueProd, 2))

    val featureSum1 = head.composePrepare { r: PMIStats[T, U, V] => r.featureStats1.featureSum }
    val featureSum2 = head.composePrepare { r: PMIStats[T, U, V] => r.featureStats2.featureSum }

    val totalAggregator = MultiAggregator(
      (
        sumProd,
        sumProdSq,
        featureSum1,
        featureSum2,
      ))

    data1WithSums
      .join(data2WithSums)
      .maybeWithReducers(optFeatureCrossReducers)
      .map {
        case (instanceId, (featureStats1, featureStats2)) =>
          val prod = featureStats1.featureValue * featureStats2.featureValue
          PMIStats(instanceId, featureStats1, featureStats2, prod)
      }
      .groupBy(pmiStats => (pmiStats.featureStats1.featureId, pmiStats.featureStats2.featureId))
      .maybeWithReducers(optFeatureCrossReducers)
      .aggregate(totalAggregator)
      .cross(distinctInstanceCount)
      .map {
        case (
              ((featureId1, featureId2), (sumProd, sumProdSq, featureSum1, featureSum2)),
              instanceCount) =>
          val rawPMI = computePMI(
            sumProd,
            featureSum1,
            featureSum2,
            instanceCount
          )

          CrossedPMIStats(
            featureId1,
            featureId2,
            sumProd,
            sumProdSq,
            featureSum1,
            featureSum2,
            instanceCount,
            rawPMI
          )
      }
  }

  def bigPMI[T, U, V](
    dataStore: PMITypes.PMIDataContainer[T, U, V],
    optFeatureCrossReducers: Option[Int] = None
  )(
    implicit orderingT: Ordering[T],
    orderingU: Ordering[U],
    serializerU: U => Array[Byte],
    orderingV: Ordering[V],
    serializerV: V => Array[Byte],
    jobParams: PMIParams,
    mathParams: MathParams,
    orderingUV: Ordering[(U, V)]
  ): TypedPipe[CrossedPMIStats[U, V]] = {
    val unSmoothedMatrix = returnUnsmoothedMatrix(dataStore, optFeatureCrossReducers)

    unSmoothedMatrix
      .flatMap { pmiStats =>
        val (
          featureId1,
          featureId2,
          sumProd,
          sumProdSq,
          featureSum1,
          featureSum2,
          totalInstances,
          _) = CrossedPMIStats.unapply(pmiStats).get

        val rawPMI = computePMIExpWithVariance(
          sumProd,
          sumProdSq,
          featureSum1,
          featureSum2,
          totalInstances
        )

        getSparsePMI(rawPMI).map(finalPMI =>
          CrossedPMIStats(
            featureId1,
            featureId2,
            sumProd,
            sumProdSq,
            featureSum1,
            featureSum2,
            totalInstances,
            finalPMI
          ))
      }
  }

  def bigPMIwithUnsmoothedMatrix[T, U, V](
    dataStore: PMITypes.PMIDataContainer[T, U, V],
    optFeatureCrossReducers: Option[Int] = None
  )(
    implicit orderingT: Ordering[T],
    orderingU: Ordering[U],
    serializerU: U => Array[Byte],
    orderingV: Ordering[V],
    serializerV: V => Array[Byte],
    jobParams: PMIParams,
    mathParams: MathParams,
    orderingUV: Ordering[(U, V)]
  ): (TypedPipe[CrossedPMIStats[U, V]], TypedPipe[CrossedPMIStats[U, V]]) = {
    val unSmoothedMatrix = returnUnsmoothedMatrix(dataStore, optFeatureCrossReducers)

    val smoothedMatrix = unSmoothedMatrix
      .flatMap { pmiStats =>
        val (
          featureId1,
          featureId2,
          sumProd,
          sumProdSq,
          featureSum1,
          featureSum2,
          totalInstances,
          _) = CrossedPMIStats.unapply(pmiStats).get

        val rawPMI = computePMIExpWithVariance(
          sumProd,
          sumProdSq,
          featureSum1,
          featureSum2,
          totalInstances
        )

        getSparsePMI(rawPMI).map(finalPMI =>
          CrossedPMIStats(
            featureId1,
            featureId2,
            sumProd,
            sumProdSq,
            featureSum1,
            featureSum2,
            totalInstances,
            finalPMI
          ))
      }

    (unSmoothedMatrix, smoothedMatrix)
  }

  def bigPMI[T, U](
    dataStore: PMITypes.SymmetricPMIContainer[T, U],
    optFeatureCrossReducers: Option[Int],
  )(
    implicit orderingT: Ordering[T],
    orderingU: Ordering[U],
    serializerU: U => Array[Byte],
    regularizationParams: RegularizationParams,
    mathParams: MathParams,
    orderingUV: Ordering[(U, U)]
  ): TypedPipe[CrossedPMIStats[U, U]] = {
    val data = dataStore.data
    val distinctInstanceCount: ValuePipe[Long] = data
      .map(_._1)
      .distinct
      .aggregate(size)

    val regularizedData = applyRegularizationToPipe(
      data,
      regularizationParams.maxFeatsPerInstance,
      regularizationParams.maxFeatSumPerInstance,
    )

    val featureSums = getFeatureSumsWithMinFeatures(
      regularizedData,
      regularizationParams.minFeatureCount,
    )

    val dataWithSums = dataStore.joinFeatureSumsToPipe(
      regularizedData,
      featureSums,
    )

    val sumProd = prepareMonoid[PMIStats[T, U, U], Double](_.valueProd)
    val sumProdSq =
      prepareMonoid[PMIStats[T, U, U], Double](pmiStats => Math.pow(pmiStats.valueProd, 2))

    val featureSum1 = head.composePrepare { r: PMIStats[T, U, U] => r.featureStats1.featureSum }
    val featureSum2 = head.composePrepare { r: PMIStats[T, U, U] => r.featureStats2.featureSum }

    val totalAggregator = MultiAggregator(
      (
        sumProd,
        sumProdSq,
        featureSum1,
        featureSum2,
      ))

    dataWithSums.group
      .maybeWithReducers(optFeatureCrossReducers)
      .toList
      .maybeWithReducers(optFeatureCrossReducers)
      .flatMap {
        case (instanceId, featureStats) =>
          featureStats
            .combinations(2)
            .map {
              case List(x, y) =>
                val (l, r) = if (x.featureId < y.featureId) (x, y) else (y, x)
                PMIStats(instanceId, l, r, l.featureValue * r.featureValue)
            }
      }
      .groupBy(pmiStats => (pmiStats.featureStats1.featureId, pmiStats.featureStats2.featureId))
      .maybeWithReducers(optFeatureCrossReducers)
      .aggregate(totalAggregator)
      .maybeWithReducers(optFeatureCrossReducers)
      .cross(distinctInstanceCount)
      .flatMap {
        case (
              ((featureId1, featureId2), (sumProd, sumProdSq, featureSum1, featureSum2)),
              instanceCount) =>
          val rawPMI = computePMIExpWithVariance(
            sumProd,
            sumProdSq,
            featureSum1,
            featureSum2,
            instanceCount
          )

          getSparsePMI(rawPMI).map(finalPMI =>
            CrossedPMIStats(
              featureId1,
              featureId2,
              sumProd,
              sumProdSq,
              featureSum1,
              featureSum2,
              instanceCount,
              finalPMI
            ))
      }
      .flatMap { p =>
        val reversedPmiStats = CrossedPMIStats(
          p.featureId2,
          p.featureId1,
          p.sumProd,
          p.sumProdSq,
          p.featureSum2,
          p.featureSum1,
          p.totalInstances,
          p.pmi
        )
        Seq(p, reversedPmiStats)
      }
  }
}
