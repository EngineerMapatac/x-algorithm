package com.twitter.botmaker.app.scarecrow.legacy

import java.util.{Set => JSet}
import com.twitter.botmaker.FunctionUnit3
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.util.Future
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.ASTNode.CacheLevel

object AddGuanoNote
    extends FunctionUnit3[ScarecrowRuntime, Long, Long, String, Future[
      JSet[com.twitter.botmaker.ResponseCode]
    ]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.PROD_OTHER_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.Guano)
  override def description = "Add a guano note."
  override def arguments: Seq[String] = Seq[String]("userId", "byUserId", "note to add")
  override def examples: Seq[String] = Seq[String]("AddGuanoNote(1111, 2222, \"test note\")")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    userId: Long,
    byUserId: Long,
    note: String
  ): Future[JSet[com.twitter.botmaker.ResponseCode]] = {
    com.twitter.botmaker.compiler.v2.astnode.AddGuanoNote
      .migrateEvaluate(
        context.getRuntime.fetcher10,
        context.getRuntime.getMigrateContext(context),
        Future.value(userId),
        Future.value(byUserId),
        Future.value(note)
      )
  }
}
