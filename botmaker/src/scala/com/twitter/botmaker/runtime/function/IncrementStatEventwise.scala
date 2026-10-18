package com.twitter.botmaker.runtime.function

import com.google.common.base.CharMatcher
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.exceptions.RuntimeFailure
import com.twitter.botmaker.rule.RuleContext
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.Context
import com.twitter.botmaker.FunctionUnit1
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.DownstreamService

object IncrementStatEventwise extends FunctionUnit1[TwitterRuntime, String, Unit] {

  override def cacheLevel: ASTNode.CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description = "Increments a stat, deduplicated across all bots on this event."
  override def arguments: Seq[String] = Seq[String]("The name of the stat")
  override def examples: Seq[String] = Seq[String]("IncrementStatEventwise(\"TestsRun\")")

  private[this] val cachedException = new RuntimeFailure(
    "IncrementStatEventwise needs an EventContext.")

  override def evaluate(context: Context[TwitterRuntime], statName: String): Unit = {
    if (CharMatcher.ascii.matchesAllOf(statName)) {
      context.getRootContext match {
        case rc: RuleContext[_] =>
          rc.eventContext.cacheGet(this, statName) {
            context.getRuntime.statsReceiver.counter(BotStats.Prefix, statName).incr()
          }
        case _ =>
          throw cachedException
      }
    }
  }
}
