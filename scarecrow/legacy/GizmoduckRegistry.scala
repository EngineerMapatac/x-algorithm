package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker._
import com.twitter.util.Future

object GetUser
    extends FunctionUnit1[ScarecrowRuntime, Long, Future[com.twitter.gizmoduck.thriftjava.User]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.Gizmoduck)
  override def description = "get gizmoduck user given a userId"
  override def arguments: Seq[String] = Seq[String]("user-id")
  override def examples: Seq[String] = Seq[String]("GetUser(123)")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    userId: Long
  ): Future[com.twitter.gizmoduck.thriftjava.User] = {
    context.getRuntime.fetcher10.getGizmoduckUser(Future.value(userId))
  }
}

object GetUserIdFromScreenName extends FunctionUnit1[ScarecrowRuntime, String, Future[Long]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.Gizmoduck)
  override def description = "Function to get userID given the user's screen name"
  override def arguments: Seq[String] = Seq[String]("user ID")
  override def examples: Seq[String] = Seq[String]("GetUserIdFromScreenName(\"testuser\")")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    screenName: String
  ): Future[Long] = {
    com.twitter.botmaker.extractor.function.GetUserIdFromScreenName
      .migrateEvaluate(
        context.getRuntime.fetcher10,
        Future.value(screenName)
      ).map(Long2long)
  }
}
