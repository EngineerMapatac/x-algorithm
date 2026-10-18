package com.twitter.botmaker.runtime.function

import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit0V

object GetHelp extends FunctionUnit0V[TwitterRuntime, String, Map[String, String]] {
  override def evaluate(
    context: Context[TwitterRuntime],
    vargs: Seq[String]
  ): Map[String, String] = {
    val runtime = context.getRuntime
    RuntimeDebugger
      .infos(
        runtime,
        vargs.toList collect {
          case x: String => x
        }
      )
  }

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def description: String = "Gets runtime helps"

  override def arguments: Seq[String] = Seq("path to help topics")
}
