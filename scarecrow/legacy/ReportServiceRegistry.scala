package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit4O2
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.useng.rtp.core.thriftjava.Entity
import com.twitter.util.Future
import java.lang.{Long => JLong}

object CreateRtpReportAsyncProd
    extends FunctionUnit4O2[ScarecrowRuntime, Entity, Long, String, Long, String, JLong, Future[
      Unit
    ]] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event

  override def actionLevel: ActionLevel = ActionLevel.PROD_AGENT_WORKFLOW_ACTION

  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.Eventbus)

  override def description: String = "Creates a report in RTP async via prod event bus."

  override def arguments: Seq[String] = Seq(
    "the entity to be reported",
    "user id to be reported",
    "report type",
    "bot id",
    "optional Note",
    "optional victim user id"
  )

  override def evaluate(
    context: Context[ScarecrowRuntime],
    entity: Entity,
    userId: Long,
    reportType: String,
    botId: Long,
    note: Option[String],
    victimId: Option[JLong]
  ): Future[Unit] = {
    context.getRuntime.fetcher20.createRtpReportAsync(
      entity,
      userId,
      reportType,
      botId,
      note,
      victimId,
      false
    )
  }
}
