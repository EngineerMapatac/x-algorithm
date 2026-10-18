package com.twitter.hub.pmi

import com.twitter.algebird.Aggregator.reservoirSample
import com.twitter.scalding.TypedPipe

object PreProcessors {

  def sampleFeaturesPerInstance[T, U](
    pipe: TypedPipe[(T, U, Double)],
    maxFeatures: Int,
    reducers: Option[Int] = None
  )(
    implicit ordering: Ordering[T]
  ): TypedPipe[(T, U, Double)] = {
    pipe
      .groupBy(_._1)
      .maybeWithReducers(reducers)
      .aggregate(reservoirSample(maxFeatures))
      .flattenValues
      .values
  }

  def capFeatureSumPerInstance[T, U](
    pipe: TypedPipe[(T, U, Double)],
    maxSumPerInstance: Int,
    reducers: Option[Int] = None
  )(
    implicit orderingT: Ordering[T]
  ): TypedPipe[(T, U, Double)] = {
    val instanceSums = pipe
      .map { case (instanceId, featureId @ _, value) => (instanceId, value) }
      .group
      .sum

    pipe
      .groupBy(_._1)
      .join(instanceSums)
      .maybeWithReducers(reducers)
      .mapValues {
        case ((instanceId, featureId, value), sumValues) =>
          val newValue = if (sumValues > maxSumPerInstance) {
            value * maxSumPerInstance / sumValues
          } else {
            value
          }
          (instanceId, featureId, newValue)
      }
      .values
  }
}
