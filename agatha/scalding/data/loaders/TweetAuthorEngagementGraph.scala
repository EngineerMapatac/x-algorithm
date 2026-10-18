package com.twitter.agatha.scalding.data.loaders

import com.twitter.agatha.scalding.data.DataLoaderUtil
import com.twitter.agatha.scalding.data.GraphBasedDataLoader
import com.twitter.hub.agatha.thriftscala.Feature
import com.twitter.hub.agatha.thriftscala.FeatureSource
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.thrift.macros.Macros._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import tweetsource.common.UnhydratedFlatScalaDataset

case class TweetAuthorEngagementGraph(createReverseFeatures: Boolean) extends GraphBasedDataLoader {
  import DataLoaderUtil._

  private[loaders] def generateEngageFeatures(
    userId: Long,
    rtAuthorIdOpt: Option[Long],
    quoteAuthorIdOpt: Option[Long],
    replyAuthorIdOpt: Option[Long],
    mentionIds: Seq[Long]
  ): Seq[(Long, Feature)] = {
    val engageInfo = (rtAuthorIdOpt, quoteAuthorIdOpt, replyAuthorIdOpt) match {
      case (Some(rtId), _, _) if userId != rtId && notTestAccount(rtId) =>
        List((userId, rtId, FeatureSource.RetweetGraph))
      case (_, Some(qtId), _) if userId != qtId && notTestAccount(qtId) =>
        List((userId, qtId, FeatureSource.QuoteGraph))
      case (_, _, Some(replyId)) if userId != replyId && notTestAccount(replyId) =>
        List((userId, replyId, FeatureSource.ReplyGraph))
      case _ => Nil
    }

    val mentionInfo = mentionIds
      .collect {
        case id if notTestAccount(id) =>
          (userId, id, FeatureSource.MentionGraph)
      }

    (engageInfo ++ mentionInfo)
      .flatMap {
        case (uId, destId, identifier) =>
          getDirectionalGraphFeaturesFromUserPair(uId, destId, identifier, createReverseFeatures)
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
          "atMentionedUserIds"
        ))
      .toTypedPipe
      .filter { tweet => notTestAccount(tweet.userId) }
      .flatMap { tweet =>
        val userId = tweet.userId
        val rtAuthorIdOpt = tweet.shareSourceUserId
        val quoteAuthorIdOpt = tweet.quotedTweetUserId
        val replyAuthorIdOpt = tweet.inReplyToUserId
        val notMentionIdSet =
          (rtAuthorIdOpt.toList ++ quoteAuthorIdOpt.toList ++ replyAuthorIdOpt.toList).toSet

        val mentionIds =
          tweet.atMentionedUserIds.toSeq.flatten.filter(id => !notMentionIdSet.contains(id))

        generateEngageFeatures(
          userId,
          rtAuthorIdOpt,
          quoteAuthorIdOpt,
          replyAuthorIdOpt,
          mentionIds
        )
      }
      .distinct
  }
}
