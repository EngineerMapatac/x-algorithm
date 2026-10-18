package com.twitter.agatha.scalding.data

import com.twitter.common_internal.analytics.artificial_user_filter.ArtificialUserFilter
import com.twitter.hub.agatha.thriftscala._
import com.twitter.scalding._
import com.twitter.scalding.serialization.RequiredBinaryComparators._

object DataLoaderUtil {

  def notTestAccount(userId: Long): Boolean = {
    !ArtificialUserFilter.isArtificialOrSignedOutId(userId)
  }

  def getDirectionalGraphFeaturesFromUserPair(
    userId: Long,
    destId: Long,
    featureSource: FeatureSource,
    createReverseFeatures: Boolean
  ): Seq[(Long, Feature)] = {
    if (userId == destId) {
      Nil
    } else {
      val outboundFeature = Seq((
        userId,
        Feature(FeatureIdentifier(FeatureClass(featureSource, "outbound"), destId.toString), 1.0)))

      val inboundFeature = {
        if (createReverseFeatures)
          Seq(
            (
              destId,
              Feature(
                FeatureIdentifier(FeatureClass(featureSource, "inbound"), userId.toString),
                1.0)))
        else
          Nil
      }

      outboundFeature ++ inboundFeature
    }
  }

  def createUserFeatures(
    dataLoaders: Set[DataLoader],
    users: TypedPipe[Long],
    dateRange: DateRange
  ): TypedPipe[UserFeature] = {
    dataLoaders
      .map(_.read(dateRange).join(users.asKeys).mapValues(_._1).toTypedPipe)
      .reduce(_ ++ _)
      .map { case (userId, feature) => UserFeature(userId, feature) }
  }
}
