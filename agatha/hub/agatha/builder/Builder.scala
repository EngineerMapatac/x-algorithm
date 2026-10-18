package com.twitter.hub.agatha.builder

import com.twitter.hub.agatha.Bijections._
import com.twitter.hub.agatha.labels.LabelManager
import com.twitter.hub.agatha.thriftscala.Feature
import com.twitter.hub.agatha.thriftscala.ModelledFeature
import com.twitter.hub.agatha.thriftscala.UserFeature
import com.twitter.hub.agatha.thriftscala.UserLabel
import com.twitter.hub.agatha.util.CrossedPMIStatToModelledFeature
import com.twitter.hub.pmi.PMI._
import com.twitter.hub.pmi.PMITypes.BigToSmallPMIContainer
import com.twitter.hub.pmi.math.MathParams
import com.twitter.scalding._
import com.twitter.scalding.thrift.macros.Macros._

object Builder {
  def apply(
    features: TypedPipe[(Long, Feature)],
    labelledUsers: TypedPipe[UserLabel],
    pmiParams: PMIParams,
    mathParams: MathParams
  )(
    implicit uniqueID: UniqueID
  ): TypedPipe[ModelledFeature] = {
    implicit val pmi: PMIParams = pmiParams
    implicit val math: MathParams = mathParams
    val convertedFeatures = features.map {
      case (uId, feature) => (uId, feature.identifier, feature.value)
    }
    val convertedLabelledUsers = labelledUsers.map(user =>
      (
        user.userId,
        user.label,
        user.smoothedRatio.getOrElse(user.positiveCount / user.totalCount.toDouble)))
    bigPMI(BigToSmallPMIContainer(convertedFeatures, convertedLabelledUsers))
      .map(CrossedPMIStatToModelledFeature(_))
  }
}

object BuilderApp {
  def apply(
    features: TypedPipe[UserFeature],
    labelManager: LabelManager,
    labelNames: Set[String],
    pmiParams: PMIParams,
    mathParams: MathParams,
    dateRange: DateRange
  )(
    implicit uniqueID: UniqueID
  ): TypedPipe[ModelledFeature] = {
    val labelMap = labelNames.map((_, dateRange)).toMap
    val labelledUsers = labelManager.getLabels(labelMap).values.reduce(_ ++ _)
    val distinctLabelUsers =
      labelledUsers.map(_.userId).distinct.withDescription("distinct label users")
    val filteredFeatures = features
      .map(uf => (uf.userId, uf.feature)).join(distinctLabelUsers.asKeys).withDescription(
        "filter features").mapValues(_._1)
    val distinctFeatureUsers =
      filteredFeatures.keys.distinct.withDescription("distinct feature users")
    val filteredUsers =
      labelledUsers.groupBy(_.userId).join(distinctFeatureUsers.asKeys).mapValues(_._1).values

    Builder(filteredFeatures, filteredUsers, pmiParams, mathParams)
  }
}
