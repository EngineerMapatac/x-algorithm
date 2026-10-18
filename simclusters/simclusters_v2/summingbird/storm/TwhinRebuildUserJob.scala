package com.twitter.simclusters_v2.summingbird.storm

import com.twitter.simclusters_v2.summingbird.common.Configs
import com.twitter.simclusters_v2.summingbird.common.Implicits
import com.twitter.simclusters_v2.summingbird.common.StatsUtil
import com.twitter.simclusters_v2.thriftscala._
import com.twitter.snowflake.id.SnowflakeId
import com.twitter.summingbird._
import com.twitter.summingbird.option.JobId
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.common.VersionId
import com.twitter.unified_user_actions.thriftscala.ActionType
import com.twitter.unified_user_actions.thriftscala.Item.TweetInfo
import com.twitter.unified_user_actions.thriftscala.UnifiedUserAction
import com.twitter.unified_user_actions.thriftscala.UserIdentifier

object TwhinRebuildUserJob {

  import Implicits._
  import StatsUtil._

  object NodeName {
    final val UUASourceNodeName: String = "UUASource"

    final val PositiveEngagementTweetEmbeddingFlatMapNodeName: String = "PosTweetEmbeddingFlatMap"
    final val PositiveEngagementUserEmbeddingSummerNodeName: String = "PosUserEmbeddingSummer"

    final val RefreshedPositiveEngagementTweetEmbeddingFlatMapNodeName: String =
      "RefreshedPosTweetEmbeddingFlatMap"
    final val RefreshedPositiveEngagementUserEmbeddingSummerNodeName: String =
      "RefreshedPosUserEmbeddingSummer"

    final val NegativeEngagementTweetEmbeddingFlatMapNodeName: String = "NegTweetEmbeddingFlatMap"
    final val NegativeEngagementUserEmbeddingSummerNodeName: String = "NegUserEmbeddingSummer"
  }

  def generate[P <: Platform[P]](
    uuaEventSource: Producer[P, UnifiedUserAction],
    tweetEmbeddingService: P#Service[(TweetId, VersionId), TwhinTweetEmbedding],
    userPositiveEmbeddingStore: P#Store[(UserId, VersionId), PersistentTwhinUserEmbedding],
    userNegativeEmbeddingStore: P#Store[(UserId, VersionId), PersistentTwhinUserEmbedding]
  )(
    implicit jobId: JobId
  ): TailProducer[P, Any] = {

    val uuaSource = uuaEventSource.name(NodeName.UUASourceNodeName)

    val twhinTweetDatasetVersionId = TwhinEmbeddingDataset.TwhinTweet.value.toLong
    val refreshedTwhinTweetDatasetVersionId = TwhinEmbeddingDataset.RefreshedTwhinTweet.value.toLong

    val qualifiedEngagementEvents = uuaSource
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
              && !isTweetTooOld(tweetInfo.actionTweetId) =>
          (tweetInfo.actionTweetId, (userId, eventMetadata.receivedTimestampMs))
        case UnifiedUserAction(
              UserIdentifier(Some(userId), _, _, _, _, _, _, _, _, _, _, _, _, _, _),
              _,
              ActionType.ClientCTALoginSuccess,
              eventMetadata,
              _,
              _) =>
          (InvalidTweetId, (userId, eventMetadata.receivedTimestampMs))
      }
      .observe("num_pos_qualified_engagement_events")

    val positiveEngagementPipe = qualifiedEngagementEvents
      .map { item =>
        ((item._1, twhinTweetDatasetVersionId), item._2)
      }
      .leftJoin(tweetEmbeddingService)
      .map {
        case ((_, versionId), ((userId, updatedAt), embedding)) =>
          (userId, versionId) ->
            PersistentTwhinUserEmbedding(
              embedding.getOrElse(persistentTwhinUserEmbeddingMonoid.zero.embedding),
              updatedAt)
      }
      .observe("pos_tweet_embedding_fetch")
      .name(NodeName.PositiveEngagementTweetEmbeddingFlatMapNodeName)
      .sumByKey(userPositiveEmbeddingStore)(persistentTwhinUserEmbeddingMonoid)
      .name(NodeName.PositiveEngagementUserEmbeddingSummerNodeName)

    val refreshedPositiveEngagementPipe = qualifiedEngagementEvents
      .map { item =>
        ((item._1, refreshedTwhinTweetDatasetVersionId), item._2)
      }
      .leftJoin(tweetEmbeddingService)
      .map {
        case ((_, versionId), ((userId, updatedAt), embedding)) =>
          (userId, versionId) ->
            PersistentTwhinUserEmbedding(
              embedding.getOrElse(persistentTwhinUserEmbeddingMonoid.zero.embedding),
              updatedAt)
      }
      .observe("refreshed_pos_tweet_embedding_fetch")
      .name(NodeName.RefreshedPositiveEngagementTweetEmbeddingFlatMapNodeName)
      .sumByKey(userPositiveEmbeddingStore)(persistentTwhinUserEmbeddingMonoid)
      .name(NodeName.RefreshedPositiveEngagementUserEmbeddingSummerNodeName)

    val qualifiedNegativeEngagementEvents = uuaSource
      .collect {
        case UnifiedUserAction(
              UserIdentifier(Some(userId), _, _, _, _, _, _, _, _, _, _, _, _, _, _),
              TweetInfo(tweetInfo),
              actionType,
              eventMetadata,
              _,
              _)
            if QualifiedNegativeEvents.contains(actionType) &&
              tweetInfo.actionTweetAuthorInfo.exists(!_.authorId.contains(userId))
              && !isTweetTooOld(tweetInfo.actionTweetId) =>
          (tweetInfo.actionTweetId, (userId, eventMetadata.receivedTimestampMs))
        case UnifiedUserAction(
              UserIdentifier(Some(userId), _, _, _, _, _, _, _, _, _, _, _, _, _, _),
              _,
              ActionType.ClientCTALoginSuccess,
              eventMetadata,
              _,
              _) =>
          (InvalidTweetId, (userId, eventMetadata.receivedTimestampMs))
      }
      .observe("num_neg_qualified_engagement_events")

    val negativeEngagementPipe = qualifiedNegativeEngagementEvents
      .map { item =>
        ((item._1, twhinTweetDatasetVersionId), item._2)
      }
      .leftJoin(tweetEmbeddingService)
      .map {
        case ((_, versionId), ((userId, updatedAt), embedding)) =>
          (userId, versionId) ->
            PersistentTwhinUserEmbedding(
              embedding.getOrElse(persistentTwhinUserEmbeddingMonoid.zero.embedding),
              updatedAt)
      }
      .observe("neg_tweet_embedding_fetch")
      .name(NodeName.NegativeEngagementTweetEmbeddingFlatMapNodeName)
      .sumByKey(userNegativeEmbeddingStore)(persistentTwhinUserNegativeEmbeddingMonoid)
      .name(NodeName.NegativeEngagementUserEmbeddingSummerNodeName)

    positiveEngagementPipe.also(refreshedPositiveEngagementPipe).also(negativeEngagementPipe)
  }

  private def isTweetTooOld(tweetId: TweetId): Boolean = {
    SnowflakeId.unixTimeMillisOptFromId(tweetId).exists { millis =>
      System.currentTimeMillis() - millis >= Configs.OldestTweetFavEventTimeInMillis
    }
  }

  private val QualifiedPositiveEvents: Set[ActionType] =
    Set(
      ActionType.ServerTweetFav,
      ActionType.ServerTweetQuote,
      ActionType.ServerTweetRetweet,
      ActionType.ClientTweetClickShare,
      ActionType.ClientTweetBookmark
    )

  private val QualifiedNegativeEvents: Set[ActionType] =
    Set(
      ActionType.ServerTweetReport,
      ActionType.ClientTweetSeeFewer,
      ActionType.ClientTweetNotInterestedIn,
      ActionType.ClientTweetBlockAuthor,
      ActionType.ClientTweetMuteAuthor
    )

  private val InvalidTweetId: TweetId = -1

}
