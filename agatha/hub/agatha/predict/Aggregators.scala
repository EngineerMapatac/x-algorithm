package com.twitter.hub.agatha.predict

import com.twitter.hub.agatha.thriftscala._
import com.twitter.hub.pmi.PMISetExtractors
import com.twitter.hub.pmi.math.aggregators.Moments
import com.twitter.hub.pmi.math.aggregators.Percentiles
import com.twitter.scalding.thrift.macros.Macros._
import com.twitter.scalding._

object Aggregators {

  def getMoments(
    features: TypedPipe[(Long, Feature)],
    pmiSet: TypedPipe[((UserGraphGrouping, String), Double)],
    numMoments: Int
  ): TypedPipe[(Long, FeatureClass, String, Map[Int, Double])] = {
    val augmentedFeatures = features.map {
      case (uId, feature) => ((uId, feature.identifier.featureClass), feature.value)
    }
    val featureCounts = PMISetExtractors.getEntityCounts(augmentedFeatures)

    Moments
      .getMoments(pmiSet, featureCounts, numMoments)
      .map {
        case (((userId, featureClass), label), centralMoments) =>
          (userId, featureClass, label, centralMoments.standardize.toSparseMap)
      }
  }

  def getPresentMoments(
    pmiSet: TypedPipe[((UserGraphGrouping, String), Double)]
  ): TypedPipe[(Long, FeatureClass, String, Map[Int, Double])] = {
    Moments
      .getPresentMoments(pmiSet)
      .map {
        case (((userId, featureClass), label), centralMoments) =>
          (userId, featureClass, label, centralMoments.standardize.toSparseMap)
      }
  }

  def getPresentPercentiles(
    pmiSet: TypedPipe[((UserGraphGrouping, String), Double)],
    percentiles: Seq[Double],
    offset: Int,
    k: Int
  ): TypedPipe[(Long, FeatureClass, String, Map[Double, Double])] = {
    Percentiles
      .getPresentPercentiles(pmiSet, percentiles, offset, k)
      .map {
        case (((userId, featureClass), label), percentileMap) =>
          (userId, featureClass, label, percentileMap)
      }
  }
}
