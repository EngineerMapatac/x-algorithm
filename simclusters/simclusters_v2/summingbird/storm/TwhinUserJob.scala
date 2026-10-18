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
import com.twitter.unified_user_actions.thriftscala.ActionType
import com.twitter.unified_user_actions.thriftscala.Item.TweetInfo
import com.twitter.unified_user_actions.thriftscala.UnifiedUserAction
import com.twitter.unified_user_actions.thriftscala.UserIdentifier

object TwhinUserJob {

  import Implicits._
  import StatsUtil._

  object NodeName {
    final val UUASourceNodeName: String = "UUASource"

    final val PositiveEngagementTweetEmbeddingFlatMapNodeName: String = "TweetEmbeddingFlatMap"
    final val PositiveEngagementUserEmbeddingSummerNodeName: String = "UserEmbeddingSummer"

    final val NegativeEngagementTweetEmbeddingFlatMapNodeName: String = "TweetEmbeddingFlatMap"
    final val NegativeEngagementUserEmbeddingSummerNodeName: String = "UserEmbeddingSummer"
  }

  def generate[P <: Platform[P]](
    uuaEventSource: Producer[P, UnifiedUserAction],
    tweetEmbeddingService: P#Service[TweetId, TwhinTweetEmbedding],
    userPositiveEmbeddingStore: P#Store[UserId, PersistentTwhinUserEmbedding],
    userNegativeEmbeddingStore: P#Store[UserId, PersistentTwhinUserEmbedding]
  )(
    implicit jobId: JobId
  ): TailProducer[P, Any] = {

    val uuaSource = uuaEventSource.name(NodeName.UUASourceNodeName)

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
      .observe("num_qualified_engagement_events")

    val positiveEngagementPipline = qualifiedEngagementEvents
      .leftJoin(tweetEmbeddingService)
      .map {
        case (_, ((userId, updatedAt), embedding)) =>
          userId ->
            PersistentTwhinUserEmbedding(
              embedding.getOrElse(persistentTwhinUserEmbeddingMonoid.zero.embedding),
              updatedAt)
      }
      .observe("tweet_embedding_fetch")
      .name(NodeName.PositiveEngagementTweetEmbeddingFlatMapNodeName)
      .sumByKey(userPositiveEmbeddingStore)(persistentTwhinUserEmbeddingMonoid)
      .name(NodeName.PositiveEngagementUserEmbeddingSummerNodeName)

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
      .observe("num_qualified_engagement_events")

    val negativeEngagementPipeline = qualifiedNegativeEngagementEvents
      .leftJoin(tweetEmbeddingService)
      .map {
        case (_, ((userId, updatedAt), embedding)) =>
          userId ->
            PersistentTwhinUserEmbedding(
              embedding.getOrElse(persistentTwhinUserEmbeddingMonoid.zero.embedding),
              updatedAt)
      }
      .observe("tweet_embedding_fetch")
      .name(NodeName.NegativeEngagementTweetEmbeddingFlatMapNodeName)
      .sumByKey(userNegativeEmbeddingStore)(persistentTwhinUserNegativeEmbeddingMonoid)
      .name(NodeName.NegativeEngagementUserEmbeddingSummerNodeName)

    positiveEngagementPipline.also(negativeEngagementPipeline)
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
