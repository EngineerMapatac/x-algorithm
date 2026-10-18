package com.twitter.agatha.scalding.data

import com.twitter.agatha.scalding.data.loaders._
import com.twitter.agatha.scalding.util.GetActiveUsers
import com.twitter.agatha.scalding.util.GetSuspendedUsers
import com.twitter.hub.agatha.thriftscala.FeatureClass
import com.twitter.hub.agatha.thriftscala.FeatureSource
import com.twitter.hub.agatha.thriftscala.UserFeature
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.Duration
import com.twitter.scalding.Execution
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.serialization.RequiredBinaryComparators._
import com.twitter.scalding.serialization.RequiredBinaryComparatorsExecutionApp
import com.twitter.scalding_internal.dalv2.DALWrite._

object ActiveUserFeaturesConfig extends FeatureExtractionConfig {
  override val historyDuration: Duration = Days(21)

  override val dataLoaders = Set(
    AbuseReportGraphDataLoader(true),
    CombinedUserDataLoader,
    FavGraphDataLoader(true),
    FollowGraphDataLoader(true),
    HashtagTweetsDataLoader,
    TweetAuthorEngagementGraph(true),
    LegitTattleGraphDataLoader(true),
    NonLegitTattlesNoBlocksGraphDataLoader(true),
    DirectMessageGraphDataLoader(true),
  )

  val experimentalGraphs: Set[FeatureClass] =
    Set(
      FeatureClass(FeatureSource.DirectMessageGraph, "inbound"),
      FeatureClass(FeatureSource.DirectMessageGraph, "outbound"),
    )

  override def getUsers(dateRange: DateRange): TypedPipe[Long] = {
    // The production job unions a third user population not in this release.
    (GetActiveUsers(dateRange) ++ GetSuspendedUsers(dateRange)).distinct
  }

  override def writeUserFeatures(
    userFeatures: TypedPipe[UserFeature],
    dateRange: DateRange
  ): Execution[Unit] = {
    userFeatures.forceToDisk
      .map(identity)
      .writeDALSnapshotExecution(
        ActiveUserFeaturesScalaDataset,
        D.Daily,
        D.Suffix("features/activeUser"),
        D.Parquet,
        dateRange.end
      )
  }
}

object ActiveUserFeaturesJob
    extends FeatureExtractionTemplate(ActiveUserFeaturesConfig)
    with RequiredBinaryComparatorsExecutionApp
object ScheduledActiveUserFeaturesJob
    extends ScheduledFeatureExtractionTemplate(ActiveUserFeaturesConfig)
