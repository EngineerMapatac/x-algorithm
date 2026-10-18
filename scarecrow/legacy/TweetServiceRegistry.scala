package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker._
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.tweetypie.thriftjava.Tweet
import com.twitter.util.Future
import java.util.{Map => JMap}

object GetTweet
    extends FunctionUnit1O1[
      ScarecrowRuntime,
      Long,
      Boolean,
      Future[Tweet]
    ] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.Tweetypie)
  override def description = "Fetches a Tweet from Tweetypie."
  override def arguments: Seq[String] =
    Seq[String](
      "The ID of the Tweet to fetch",
      "Optional param (defaults to FALSE): Set to true if an attempt to fetch a deleted Tweet should be made if a Tweet with the given ID was not found. The ID (result.id) will be set to -1 to indicate that the Tweet was deleted."
    )
  override def examples: Seq[String] = Seq[String]("GetTweet(123)")

  override def evaluate(
    context: Context[ScarecrowRuntime],
    tweetId: Long,
    fetchDeletedTweetIfNotFound: Option[Boolean]
  ): Future[Tweet] = {
    com.twitter.botmaker.compiler.v2.astnode.GetTweet.migrateEvaluate(
      context.getRuntime.fetcher10,
      context.getRuntime.getMigrateContext(context),
      Future.value(tweetId),
      Future.value(Boolean.box(fetchDeletedTweetIfNotFound.getOrElse(false)))
    )
  }
}

object GetTweetInfo extends FunctionUnit1[ScarecrowRuntime, Long, Future[JMap[String, AnyRef]]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.Tweetypie)
  override def description =
    "Given a tweet ID, returns a map of tweet info. Example:\n{ id=561234161692708865, \ntext=hello #world, \ncreated_at=1422643626, \ncreated_via=oauth:8719, \nconversation_id=561234161692708865, \nlanguage=Language(language:es, right_to_left:false, confidence:0.35828420519828796), \nmentions=[], \nhashtags=[world]}"
  override def arguments: Seq[String] = Seq[String]("the tweet id")
  override def examples: Seq[String] = Seq[String]("GetTweetInfo(123456789)")

  override def evaluate(
    context: Context[ScarecrowRuntime],
    tweetId: Long
  ): Future[JMap[String, AnyRef]] = {
    com.twitter.botmaker.compiler.v2.astnode.GetTweetInfo.migrateEvaluate(
      context.getRuntime.fetcher10,
      context.getRuntime.getMigrateContext(context),
      Future.value(tweetId)
    )
  }
}

object GetTweetText extends FunctionUnit1[ScarecrowRuntime, Long, Future[String]] {
  val tweetTextTruncatedLengthDeciderKey = "tweet_text_truncated_length"
  val tweetTextMaxLengthDeciderKey = "tweet_text_max_length"

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.Tweetypie)
  override def description = "The text of a tweet."
  override def arguments: Seq[String] = Seq[String]("the tweet id")
  override def examples: Seq[String] = Seq[String]("GetTweetText(123456789)")

  override def evaluate(context: Context[ScarecrowRuntime], tweetId: Long): Future[String] = {
    com.twitter.botmaker.extractor.function.GetTweetText.migrateEvaluate(
      context.getRuntime.fetcher10,
      context.getRuntime.getMigrateContext(context),
      Future.value(tweetId)
    )
  }
}
