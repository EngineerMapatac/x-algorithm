package com.twitter.hub.pmi

import com.twitter.algebird.Aggregator.prepareMonoid
import com.twitter.algebird.Aggregator.size
import com.twitter.algebird.MultiAggregator
import com.twitter.hub.pmi.PreProcessors._
import com.twitter.scalding.TypedPipe

private[pmi] object ModelCommon {
  def applyRegularizationToPipe[T, U](
    pipe: TypedPipe[(T, U, Double)],
    maxFeatsPerInstance: Option[Int],
    maxFeatSumPerInstance: Option[Int]
  )(
    implicit orderingT: Ordering[T]
  ): TypedPipe[(T, U, Double)] = {
    val sampledPipe = maxFeatsPerInstance
      .map(m => sampleFeaturesPerInstance(pipe, m))
      .getOrElse(pipe)

    val cappedPipe = maxFeatSumPerInstance
      .map(m => capFeatureSumPerInstance(sampledPipe, m))
      .getOrElse(sampledPipe)

    cappedPipe
  }

  def getFeatureSumsWithMinFeatures[T, U](
    pipe: TypedPipe[(T, U, Double)],
    minFeatureCount: Option[Int]
  )(
    implicit ordering: Ordering[U]
  ): TypedPipe[(U, Double)] = {

    val sizeAndSum = MultiAggregator(
      (
        size,
        prepareMonoid[Double, Double](x => x)
      ))

    pipe
      .map { case (instanceId @ _, featureId, value) => (featureId, value) }
      .group
      .aggregate(sizeAndSum)
      .collect {
        case (featureId, (size, sum)) if size >= minFeatureCount.getOrElse(0) =>
          (featureId, sum)
      }
  }

  def getDataWithSumsSketch[T, U](
    regularizedPipe: TypedPipe[(T, U, Double)],
    featureSums: TypedPipe[(U, Double)],
    reducers: Int = 5000
  )(
    implicit ordering: Ordering[U],
    serializer: U => Array[Byte]
  ): TypedPipe[(T, PMI.FeatureStats[U])] = {
    regularizedPipe
      .groupBy(_._2)
      .sketch(reducers)
      .join(featureSums)
      .mapValues {
        case ((instanceId, featureId, value), featureSum) =>
          (instanceId, PMI.FeatureStats(featureId, value, featureSum))
      }
      .values
  }

  def getDataWithSumsHash[T, U](
    regularizedPipe: TypedPipe[(T, U, Double)],
    featureSums: TypedPipe[(U, Double)]
  )(
    implicit ordering: Ordering[U]
  ): TypedPipe[(T, PMI.FeatureStats[U])] = {
    regularizedPipe
      .groupBy(_._2)
      .hashJoin(featureSums)
      .mapValues {
        case ((instanceId, featureId, value), featureSum) =>
          (instanceId, PMI.FeatureStats(featureId, value, featureSum))
      }
      .values
  }
}
