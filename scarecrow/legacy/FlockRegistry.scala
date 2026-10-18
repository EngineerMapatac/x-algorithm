package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit1O1
import com.twitter.botmaker.FunctionUnit2
import com.twitter.util.Future
import java.lang.{Boolean => JBoolean}
import java.lang.{Long => JLong}

object FollowerCount extends FunctionUnit1O1[ScarecrowRuntime, Long, JBoolean, Future[JLong]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.SocialGraphService)
  override def description = "The given user's follower count."
  override def arguments: Seq[String] =
    Seq[String]("the user ID", "whether include inactive edge (default = FALSE)")
  override def examples: Seq[String] = Seq[String]("FollowerCount(123, TRUE)")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    userId: Long,
    withInactive: Option[JBoolean]
  ): Future[JLong] = {
    com.twitter.botmaker.compiler.v2.astnode.FollowerCount.migrateEvaluate(
      context.getRuntime.fetcher10,
      Future.value(userId),
      Future.value(withInactive.getOrElse(false))
    )
  }
}

object IsFollowing extends FunctionUnit2[ScarecrowRuntime, Long, Long, Future[Boolean]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.SocialGraphService)
  override def description = "Checks Flock to see if follower is following followed."
  override def arguments: Seq[String] =
    Seq[String](
      "FollowerId the user Id of the follower",
      "FollowedId the userId of the user who is being followed"
    )
  override def examples: Seq[String] = Seq[String]("IsFollowing(123, 456)")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    followerId: Long,
    userBeingFollowedId: Long
  ): Future[Boolean] = {
    com.twitter.botmaker.extractor.function.IsFollowing
      .migrateEvaluate(
        context.getRuntime.fetcher10,
        Future.value(followerId),
        Future.value(userBeingFollowedId)
      ).map(Boolean2boolean)
  }
}
