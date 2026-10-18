package com.twitter.agatha.scalding.labels

import com.twitter.agatha.scalding.util.GetActiveUsers
import com.twitter.algebird.Aggregator.reservoirSample
import com.twitter.hub.agatha.thriftscala.UserLabel
import com.twitter.scalding.serialization.RequiredBinaryComparators._
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.ValuePipe

object LabelUtil {
  def getLabelSizes(
    labelPipe: TypedPipe[(Long, Long)],
    maxTakePerSourceID: Int = 100
  ): TypedPipe[(Long, Long)] = {
    labelPipe
      .groupBy(_._2)
      .aggregate(reservoirSample(maxTakePerSourceID))
      .flattenValues
      .values
      .group
      .size
  }

  def capFeatureSum(
    featurePipe: TypedPipe[(Long, Long)],
    maxFeatureSum: Double = 1000.0
  ): TypedPipe[(Double, Double)] = {
    featurePipe
      .map {
        case (positiveClass, negativeClass) =>
          val total = (positiveClass + negativeClass).toDouble
          if (total > maxFeatureSum) {
            (positiveClass * maxFeatureSum / total, negativeClass * maxFeatureSum / total)
          } else {
            (positiveClass, negativeClass)
          }
      }
  }

  def getPrior(positiveNegativePipe: TypedPipe[(Double, Double)]): ValuePipe[Double] = {
    positiveNegativePipe.sum
      .map { case (positiveSum, negativeSum) => positiveSum / (positiveSum + negativeSum) }
  }

  def createPositiveNegativeLabelsFromActiveUsers(
    positives: TypedPipe[Long],
    labelName: String,
    dateRange: DateRange
  ): TypedPipe[UserLabel] = {
    val activeUsers = GetActiveUsers(dateRange)

    val negativeUsers = activeUsers.asKeys
      .leftJoin(positives.asKeys)
      .collect { case (userId, (_, opt)) if opt.isEmpty => (userId, 0) }

    val positiveUsers = positives.map(userId => (userId, 1))

    (negativeUsers ++ positiveUsers)
      .map { case (userId, label) => UserLabel(userId, labelName, label, 1, None) }
  }
}
