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

object HashtagTweetsDataLoader extends DataLoader {
  import DataLoaderUtil._

  private[data] def processTweetData(
    userId: Long,
    rtAuthorIdOpt: Option[Long],
    hashtags: Seq[String]
  ): Seq[(Long, Feature)] = {

    val featureSource = rtAuthorIdOpt match {
      case Some(rtId) if userId != rtId =>
        FeatureSource.HashtagRetweet
      case _ => FeatureSource.HashtagOriginal
    }

    hashtags
      .map { hashtag =>
        (
          userId,
          Feature(FeatureIdentifier(FeatureClass(featureSource, ""), hashtag.toLowerCase), 1.0))
      }
  }

  override def read(dateRange: DateRange): TypedPipe[(Long, Feature)] = {

    DAL
      .read(UnhydratedFlatScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .withColumns(
        Set(
          "userId",
          "shareSourceUserId",
          "quotedTweetUserId",
          "inReplyToUserId",
          "hashtags"
        ))
      .toTypedPipe
      .filter { tweet => notTestAccount(tweet.userId) }
      .flatMap { tweet =>
        val userId = tweet.userId
        val rtAuthorIdOpt = tweet.shareSourceUserId
        val hashTags = tweet.hashtags.toSeq.flatten

        processTweetData(userId, rtAuthorIdOpt, hashTags)
      }
      .distinct
  }
}
