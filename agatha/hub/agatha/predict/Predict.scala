package com.twitter.hub.agatha.predict

import com.twitter.bijection.scrooge.BinaryScalaCodec
import com.twitter.hub.agatha.labels.LabelManager
import com.twitter.hub.agatha.thriftscala._
import com.twitter.hub.agatha.util.CrossedPMIStatToModelledFeature
import com.twitter.hub.pmi.PMISetExtractors
import com.twitter.hub.pmi.math.MathParams
import com.twitter.scalding.thrift.macros.Macros._
import com.twitter.scalding.DateRange
import com.twitter.scalding.RichDate
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.UniqueID

object Predict {

  def getPMIForUserFeatures(
    features: TypedPipe[(Long, Feature)],
    models: TypedPipe[ModelledFeature],
    labelledUsers: TypedPipe[UserLabel],
    mathParams: MathParams
  )(
    implicit uniqueID: UniqueID
  ): TypedPipe[((UserGraphGrouping, String), Double)] = {
    implicit val mp: MathParams = mathParams

    val convertedLabelledUsers = labelledUsers.map(user =>
      (
        user.userId,
        user.label,
        user.smoothedRatio.getOrElse(user.positiveCount / user.totalCount.toDouble)))
    val convertedModels = models.map(CrossedPMIStatToModelledFeature.invert)

    val augmentedFeatures = features.map {
      case (uId, feature) => (uId, feature.identifier, feature.value)
    }
    implicit val featureIdentifierToArray: FeatureIdentifier => Array[Byte] =
      BinaryScalaCodec(FeatureIdentifier).apply(_)
    PMISetExtractors
      .getLeaveOneOutPMIExpSetPerEntityAndLabel(
        augmentedFeatures,
        convertedLabelledUsers,
        convertedModels,
        None)
      .map { pmiSetOutput =>
        (
          ((pmiSetOutput.dataRow._1, pmiSetOutput.dataRow._2.featureClass), pmiSetOutput.label),
          pmiSetOutput.pmi)
      }
  }
}

object PredictionApp {
  def apply(
    features: TypedPipe[UserFeature],
    modelMetadataMap: Map[TypedPipe[ModelledFeature], ModelledFeatureLabelMetadata],
    labelManager: LabelManager,
    mathParams: MathParams
  )(
    implicit uniqueID: UniqueID
  ): TypedPipe[((UserGraphGrouping, String), Double)] = {
    val flattenedFeatures = features.map(uf => (uf.userId, uf.feature))

    val metadataCheckMap = modelMetadataMap.values
      .flatMap(_.labelDateRangeMap.toList)
      .groupBy(_._1)
      .mapValues(_.map(_._2).toSet)
      .filter { case (_, set) => set.size > 1 }

    if (metadataCheckMap.nonEmpty)
      throw new IllegalArgumentException(
        s"The following labels have multiple dateRanges: $metadataCheckMap")

    val labelMap = modelMetadataMap.values
      .map(
        _.labelDateRangeMap
          .mapValues(dr => DateRange(RichDate(dr.startTimestamp), RichDate(dr.endTimestamp))).toMap)
      .reduce(_ ++ _)
    val labels = labelManager.getLabels(labelMap).values.reduce(_ ++ _)

    val models = modelMetadataMap.keys.reduce(_ ++ _)
    Predict.getPMIForUserFeatures(flattenedFeatures, models, labels, mathParams)
  }
}
