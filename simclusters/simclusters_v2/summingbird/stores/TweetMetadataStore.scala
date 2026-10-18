package com.twitter.simclusters_v2.summingbird.stores

import com.twitter.frigate.common.store.strato.StratoFetchableStore
import com.twitter.mediaservices.commons.tweetmedia.thriftscala.MediaInfo
import com.twitter.mediaservices.commons.tweetmedia.thriftscala.MediaSizeType
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.thriftscala.SimClusterEntity
import com.twitter.spam.rtf.thriftscala.SafetyLevel
import com.twitter.storehaus.ReadableStore
import com.twitter.strato.client.Client
import com.twitter.strato.thrift.ScroogeConvImplicits._
import com.twitter.tweetypie.thriftscala.GetTweetFieldsOptions
import com.twitter.tweetypie.thriftscala.Tweet
import com.twitter.tweetypie.thriftscala.TweetCoreData
import com.twitter.tweetypie.thriftscala.TweetInclude

object TweetMetadataStore {

  def isHighQualityVideo(tweet: Tweet): Boolean = {
    val coreData = tweet.coreData

    val isReply = coreData.exists(_.reply.nonEmpty)

    val hasMultipleMedia = {
      tweet.mediaKeys.exists(_.map(_.mediaCategory).size > 1)
    }

    val hasVideo = tweet.media.exists {
      _.exists {
        _.mediaInfo match {
          case Some(MediaInfo.VideoInfo(_)) => true
          case _ => false
        }
      }
    }

    val mediaDimensionsOpt =
      tweet.media.flatMap(
        _.headOption.flatMap(_.sizes
          .find(_.sizeType == MediaSizeType.Orig).map(size => (size.width, size.height))))

    val mediaWidth = mediaDimensionsOpt.map(_._1).getOrElse(1)
    val mediaHeight = mediaDimensionsOpt.map(_._2).getOrElse(1)
    val isHighMediaResolution = mediaHeight > 480 && mediaWidth > 480

    val hasUrl = tweet.urls.exists(_.nonEmpty)

    !isReply && !hasMultipleMedia && !hasUrl && hasVideo && isHighMediaResolution
  }

  def tweetMetadataStore(
    stratoClient: Client,
    column: String
  ): ReadableStore[SimClusterEntity, TweetMetadata] = {
    StratoFetchableStore
      .withView[TweetId, GetTweetFieldsOptions, Tweet](stratoClient, column, getTweetOptions)
      .composeKeyMapping[SimClusterEntity] {
        case SimClusterEntity.TweetId(tweetId) => tweetId
      }
      .mapValues { tweet =>
        TweetMetadata(
          isHighMediaResolution = isHighQualityVideo(tweet),
          authorId = tweet.coreData.map(_.userId)
        )
      }
  }

  private val getTweetOptions = GetTweetFieldsOptions(
    tweetIncludes = Set[TweetInclude](
      TweetInclude.TweetFieldId(Tweet.MediaKeysField.id),
      TweetInclude.TweetFieldId(Tweet.MediaField.id),
      TweetInclude.TweetFieldId(Tweet.CoreDataField.id),
      TweetInclude.TweetFieldId(Tweet.UrlsField.id)
    ),
    safetyLevel = Some(SafetyLevel.Recommendations)
  )
}

case class TweetMetadata(
  isHighMediaResolution: Boolean,
  authorId: Option[Long])
