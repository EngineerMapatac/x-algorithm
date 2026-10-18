package com.twitter.botmaker.app.scarecrow.legacy

import java.net.UnknownHostException
import com.twitter.botmaker._
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.ASTNode.CacheLevel

object GetShardHostName extends FunctionUnit0[ScarecrowRuntime, String] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description = "Returns the hostname of the shard of this botmaker instance."
  override def arguments: Seq[String] = Seq[String]("")
  override def examples: Seq[String] = Seq[String]("GetShardHostName()")

  override def evaluate(context: com.twitter.botmaker.Context[ScarecrowRuntime]): String = {
    try {
      context.getRuntime.fetcher10.getShardHostName
    } catch {
      case _: UnknownHostException => "unknown"
    }
  }
}
