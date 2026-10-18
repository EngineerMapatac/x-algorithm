package com.twitter.agatha.scalding.data.loaders

import com.twitter.agatha.scalding.data.DataLoader
import com.twitter.agatha.scalding.data.DataLoaderUtil
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
import tweetsource.common.UnhydratedFlatScalaDataset

object RetweetedTweetsDataLoader extends DataLoader {
  import DataLoaderUtil._

  override def read(dateRange: DateRange): TypedPipe[(Long, Feature)] = {
    DAL
      .read(UnhydratedFlatScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .withColumns(Set("userId", "shareSourceTweetId", "shareSourceUserId"))
      .toTypedPipe
      .filter { tweet => notTestAccount(tweet.userId) }
      .flatMap { tweet =>
        for {
          rtId <- tweet.shareSourceTweetId
          rtAuthorId <- tweet.shareSourceUserId
          if notTestAccount(rtAuthorId)
        } yield (
          tweet.userId,
          Feature(
            FeatureIdentifier(
              FeatureClass(FeatureSource.RetweetedTweet, ""),
              rtId.toString
            ),
            1.0))
      }
      .distinct
  }
}
