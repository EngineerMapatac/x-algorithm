package com.twitter.simclusters_v2.summingbird.storm

import com.twitter.logging.Logger
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.common.VersionId
import com.twitter.simclusters_v2.summingbird.common.Configs
import com.twitter.simclusters_v2.summingbird.common.Implicits
import com.twitter.simclusters_v2.summingbird.common.StatsUtil
import com.twitter.simclusters_v2.thriftscala._
import com.twitter.snowflake.id.SnowflakeId
import com.twitter.summingbird._
import com.twitter.summingbird.option.JobId
import com.twitter.unified_user_actions.thriftscala.ActionType
import com.twitter.unified_user_actions.thriftscala.Item.TweetInfo
import com.twitter.unified_user_actions.thriftscala.UnifiedUserAction
import com.twitter.unified_user_actions.thriftscala.UserIdentifier

object AdsTwhinRebuildTweetJob {

  import Implicits._
  import StatsUtil._

  @transient val log: Logger = Logger.get()

  object NodeName {
    final val UUASourceNodeName: String = "UUASource"

    final val TweetUserEmbeddingFlatMapNodeName: String = "TweetUserEmbeddingFlatMap"
    final val TweetEmbeddingSummerNodeName: String = "TweetEmbeddingSummer"
    final val KeyedTweetEmbeddingMapNodeName: String = "KeyedTweetEmbeddingMap"
    final val KeyedPersistentTwhinTweetEmbeddingWriteNodeName: String =
      "KeyedPersistentTwhinTweetEmbeddingWrite"

    final val RefreshedUserEmbeddingFlatMapNodeName: String = "RefreshedUserEmbeddingFlatMap"
    final val RefreshedTweetEmbeddingSummerNodeName: String = "RefreshedTweetEmbeddingSummer"
    final val RefreshedKeyedTweetEmbeddingMapNodeName: String = "RefreshedKeyedTweetEmbeddingMap"
    final val RefreshedKeyedPersistentTwhinTweetEmbeddingWriteNodeName: String =
      "RefreshedKeyedPersistentTwhinTweetEmbeddingWrite"
  }

  def generate[P <: Platform[P]](
    uuaEventSource: Producer[P, UnifiedUserAction],
    userEmbeddingService: P#Service[UserId, TwhinTweetEmbedding],
    refreshedUserEmbeddingService: P#Service[UserId, TwhinTweetEmbedding],
    tweetEmbeddingStore: P#Store[(TweetId, VersionId), PersistentTwhinTweetEmbedding]
  )(
    implicit jobId: JobId
  ): TailProducer[P, Any] = {

    val uuaSource = uuaEventSource.name(NodeName.UUASourceNodeName)

    val twhinTweetDataset = TwhinEmbeddingDataset.TwhinTweet
    val refreshedTwhinTweetDataset = TwhinEmbeddingDataset.RefreshedTwhinTweet

    val twhinTweetDatasetVersionId = twhinTweetDataset.value.toLong
    val refreshedTwhinTweetDatasetVersionId = refreshedTwhinTweetDataset.value.toLong

    val qualifiedFavEvents = uuaSource
      .collect {
        case UnifiedUserAction(
              UserIdentifier(Some(userId), _, _, _, _, _, _, _, _, _, _, _, _, _, _),
              TweetInfo(tweetInfo),
              actionType,
              eventMetadata,
              _,
              _)
            if QualifiedPositiveEvents.contains(actionType) &&
              tweetInfo.actionTweetAuthorInfo.exists(!_.authorId.contains(userId))
              && !isTweetTooOld(tweetInfo.actionTweetId)
              && tweetInfo.promotedId.isDefined =>
          (
            userId,
            (
              tweetInfo.actionTweetId,
              tweetInfo.actionTweetAuthorInfo.flatMap(_.authorId),
              eventMetadata.receivedTimestampMs))
      }
      .observe("num_qualified_favorite_events")

    val tweetEmbeddingPipe = qualifiedFavEvents
      .leftJoin(userEmbeddingService)
      .collect {
        case (_, ((tweetId, authorId, updatedAt), Some(embedding))) =>
          (tweetId, twhinTweetDatasetVersionId) ->
            PersistentTwhinTweetEmbedding(
              embedding,
              1,
              updatedAt,
              authorId,
              Some(twhinTweetDataset))
      }
      .observe("user_embedding_fetch")
      .name(NodeName.TweetUserEmbeddingFlatMapNodeName)
      .sumByKey(tweetEmbeddingStore)
      .name(NodeName.TweetEmbeddingSummerNodeName)

    val refreshedTweetEmbeddingPipe = qualifiedFavEvents
      .leftJoin(refreshedUserEmbeddingService).collect {
        case (_, ((tweetId, authorId, updatedAt), Some(embedding))) =>
          (tweetId, refreshedTwhinTweetDatasetVersionId) ->
            PersistentTwhinTweetEmbedding(
              embedding,
              1,
              updatedAt,
              authorId,
              Some(refreshedTwhinTweetDataset)
            )
      }
      .observe("refreshed_user_embedding_fetch")
      .name(NodeName.RefreshedUserEmbeddingFlatMapNodeName)
      .sumByKey(tweetEmbeddingStore)
      .name(NodeName.RefreshedTweetEmbeddingSummerNodeName)

    tweetEmbeddingPipe.also(refreshedTweetEmbeddingPipe)
  }

  private def isTweetTooOld(tweetId: TweetId): Boolean = {
    SnowflakeId.unixTimeMillisOptFromId(tweetId).exists { millis =>
      System.currentTimeMillis() - millis >= Configs.AdsOldestTweetEventTimeInMillis
    }
  }

  private val QualifiedPositiveEvents: Set[ActionType] =
    Set(
      ActionType.ClientTweetVideoCtaUrlClick,
      ActionType.ClientTweetOpenLink,
      ActionType.ClientTweetClickShare,
      ActionType.ClientTweetBookmark,
      ActionType.ClientTweetFav,
      ActionType.ServerPromotedTweetClick,
      ActionType.ServerPromotedTweetLingerImpressionLong
    )
}
