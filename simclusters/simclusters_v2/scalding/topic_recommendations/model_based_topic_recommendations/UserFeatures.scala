package com.twitter.simclusters_v2.scalding.topic_recommendations.model_based_topic_recommendations

import com.twitter.ml.api.Feature
import com.twitter.ml.api.FeatureContext
import com.twitter.ml.api.constant.SharedFeatures

object UserFeatures {
  val UserIdFeature = SharedFeatures.USER_ID

  val UserSimClusterFeatures =
    new Feature.SparseContinuous(
      "user.simclusters.interested_in"
    )

  val UserCountryFeature = new Feature.Text("user.country")

  val UserLanguageFeature = new Feature.Text("user.language")

  val FollowedTopicIdFeatures =
    new Feature.SparseBinary(
      "followed_topics.id"
    )

  val NotInterestedTopicIdFeatures =
    new Feature.SparseBinary(
      "not_interested_topics.id"
    )

  val FollowedTopicSimClusterAvgFeatures =
    new Feature.SparseContinuous(
      "followed_topics.simclusters.avg"
    )

  val NotInterestedTopicSimClusterAvgFeatures =
    new Feature.SparseContinuous(
      "not_interested_topics.simclusters.avg"
    )

  val TargetTopicIdFeatures = new Feature.Discrete("target_topic.id")

  val TargetTopicSimClustersFeature =
    new Feature.SparseContinuous(
      "target_topic.simclusters"
    )

  val FeatureContext = new FeatureContext(
    UserIdFeature,
    UserSimClusterFeatures,
    UserCountryFeature,
    UserLanguageFeature,
    FollowedTopicIdFeatures,
    NotInterestedTopicIdFeatures,
    FollowedTopicSimClusterAvgFeatures,
    NotInterestedTopicSimClusterAvgFeatures,
    TargetTopicIdFeatures,
    TargetTopicSimClustersFeature
  )
}
