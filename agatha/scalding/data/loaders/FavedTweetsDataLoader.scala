package com.twitter.agatha.scalding.data.loaders

import com.twitter.agatha.scalding.data.DataLoader
import com.twitter.agatha.scalding.data.DataLoaderUtil.notTestAccount
import com.twitter.hub.agatha.thriftscala.Feature
import com.twitter.hub.agatha.thriftscala.FeatureClass
import com.twitter.hub.agatha.thriftscala.FeatureIdentifier
import com.twitter.hub.agatha.thriftscala.FeatureSource
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.thrift.macros.Macros._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.timelineservice.thriftscala.FavoriteEventUnion.Favorite
import twadoop_config.configuration.log_categories.group.timeline.TimelineServiceFavoritesScalaDataset

object FavedTweetsDataLoader extends DataLoader {
  override def read(dateRange: DateRange): TypedPipe[(Long, Feature)] = {
    DAL
      .read(TimelineServiceFavoritesScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .map(_.event)
      .collect {
        case Favorite(e)
            if notTestAccount(e.userId) && notTestAccount(e.tweetUserId)
              && e.userId != e.tweetUserId =>
          (
            e.userId,
            Feature(
              FeatureIdentifier(FeatureClass(FeatureSource.FavedTweet, ""), e.tweetId.toString),
              1.0)
          )
      }
      .distinct
  }
}
