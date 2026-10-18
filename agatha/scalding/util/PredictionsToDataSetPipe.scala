package com.twitter.agatha.scalding.util

import com.twitter.agatha.thriftscala.UserPrediction
import com.twitter.agatha.thriftscala.UserQuantilePrediction
import com.twitter.ml.api.Feature.Continuous
import com.twitter.ml.api.constant.SharedFeatures
import com.twitter.ml.api.util.FDsl._
import com.twitter.ml.api.DataRecord
import com.twitter.ml.api.DataRecordMerger
import com.twitter.ml.api.DataSetPipe
import com.twitter.ml.api.Feature
import com.twitter.ml.api.FeatureContext
import com.twitter.scalding.thrift.macros.Macros._
import com.twitter.scalding.Execution
import com.twitter.scalding.TypedPipe

object PredictionsToDataSetPipe {
  private val baseFeatureContext = new FeatureContext(
    SharedFeatures.USER_ID,
  )

  private def featuresToSparseMapExec[K](
    features: TypedPipe[(K, Feature.Continuous)],
    description: String
  ): Execution[Map[K, Feature.Continuous]] = {
    features.groupAll
      .withDescription(description)
      .toList
      .values
      .toIterableExecution
      .map(_.headOption.getOrElse(Nil).toMap)
  }

  def apply(
    moments: TypedPipe[UserPrediction],
    quantiles: TypedPipe[UserQuantilePrediction]
  ): Execution[DataSetPipe] = {
    val momentFeatures = moments
      .flatMap(up => up.moments.keys.map(moment => (up.label, up.featureClass, moment)))
      .distinct
      .map {
        case (label, fc, moment) =>
          val feature = fc.source.name
          val subclass = if (fc.subclass.isEmpty) "none" else fc.subclass
          (label, feature, subclass, moment) -> new Continuous(
            s"agatha.$label.$feature.$subclass.moment_$moment")
      }

    val momentSparseMapExecution =
      featuresToSparseMapExec(momentFeatures, "generating moment feature names for data record")

    val quantileFeatures = quantiles
      .flatMap(up => up.quantiles.keys.map(quantile => (up.label, up.featureClass, quantile)))
      .distinct
      .map {
        case (label, fc, quantile) =>
          val feature = fc.source.name
          val subclass = if (fc.subclass.isEmpty) "none" else fc.subclass
          (label, feature, subclass, quantile) -> new Continuous(
            s"agatha.$label.$feature.$subclass.quantile_$quantile")
      }

    val quantileSparseMapExecution =
      featuresToSparseMapExec(quantileFeatures, "generating quantile feature names for data record")

    momentSparseMapExecution.zip(quantileSparseMapExecution).map {
      case (momentSparseMap, quantileSparseMap) =>
        val featureNames = (momentSparseMap.values ++ quantileSparseMap.values).toSeq
        val featureContext = baseFeatureContext.addFeatures(featureNames: _*)

        val momentRecords = moments
          .groupBy(up => up.userId)
          .mapGroup {
            case (userId, userPredictions) =>
              val record = new DataRecord()
              record.setFeatureValue(SharedFeatures.USER_ID, userId)
              userPredictions.foreach { prediction =>
                val featureClass = prediction.featureClass
                val label = prediction.label
                val source = featureClass.source.name
                val subclass = if (featureClass.subclass.isEmpty) "none" else featureClass.subclass
                prediction.moments.foreach {
                  case (momentNum, momentValue) =>
                    record.setFeatureValue(
                      momentSparseMap((label, source, subclass, momentNum)),
                      momentValue)
                }
              }
              Iterator((userId, record))
          }.values

        val quantileRecords = quantiles
          .groupBy(up => up.userId)
          .mapGroup {
            case (userId, userPredictions) =>
              val record = new DataRecord()
              record.setFeatureValue(SharedFeatures.USER_ID, userId)
              userPredictions.foreach { prediction =>
                val featureClass = prediction.featureClass
                val label = prediction.label
                val source = featureClass.source.name
                val subclass = if (featureClass.subclass.isEmpty) "none" else featureClass.subclass
                prediction.quantiles.foreach {
                  case (quantile, quantileValue) =>
                    record.setFeatureValue(
                      quantileSparseMap((label, source, subclass, quantile)),
                      quantileValue)
                }
              }
              Iterator((userId, record))
          }.values

        val records = (momentRecords ++ quantileRecords).group.mapValueStream { dataRecords =>
          val DataRecordMerger = new DataRecordMerger
          val mergedRecord = new DataRecord()
          dataRecords.foreach(dr => DataRecordMerger.merge(mergedRecord, dr))
          Iterator(mergedRecord)
        }.values

        DataSetPipe(records, featureContext)
    }
  }
}
