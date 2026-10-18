package com.twitter.simclusters_v2.scalding.topic_recommendations.model_based_topic_recommendations

import com.twitter.ml.api.util.FDsl._
import com.twitter.ml.api.DataRecord
import com.twitter.ml.api.FeatureContext
import com.twitter.ml.api.IRecordOneToOneAdapter

case class UserTopicTrainingSample(
  userId: Long,
  followedTopics: Set[Long],
  notInterestedTopics: Set[Long],
  userCountry: String,
  userLanguage: String,
  targetTopicId: Int,
  userInterestedInSimClusters: Map[Int, Double],
  followedTopicsSimClusters: Map[Int, Double],
  notInterestedTopicsSimClusters: Map[Int, Double])

class UserTopicDataRecordAdapter extends IRecordOneToOneAdapter[UserTopicTrainingSample] {
  import UserFeatures._

  override def getFeatureContext: FeatureContext = UserFeatures.FeatureContext

  override def adaptToDataRecord(record: UserTopicTrainingSample): DataRecord = {
    val dr = new DataRecord()

    dr.setFeatureValue(UserIdFeature, record.userId)
    dr.setFeatureValue(
      UserSimClusterFeatures,
      record.userInterestedInSimClusters.map {
        case (id, score) => id.toString -> score
      })
    dr.setFeatureValue(FollowedTopicIdFeatures, record.followedTopics.map(_.toString))
    dr.setFeatureValue(NotInterestedTopicIdFeatures, record.notInterestedTopics.map(_.toString))
    dr.setFeatureValue(UserCountryFeature, record.userCountry)
    dr.setFeatureValue(UserLanguageFeature, record.userLanguage)

    dr.setFeatureValue(
      FollowedTopicSimClusterAvgFeatures,
      record.followedTopicsSimClusters.map {
        case (id, score) => id.toString -> score
      })

    dr.setFeatureValue(
      NotInterestedTopicSimClusterAvgFeatures,
      record.notInterestedTopicsSimClusters.map {
        case (id, score) => id.toString -> score
      })
    dr.setFeatureValue(TargetTopicIdFeatures, record.targetTopicId.toLong)
    dr.getRecord
  }
}
