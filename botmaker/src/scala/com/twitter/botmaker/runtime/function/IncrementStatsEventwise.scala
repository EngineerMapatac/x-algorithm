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

object IncrementStatsEventwise extends FunctionUnit1[TwitterRuntime, List[String], Unit] {

  override def cacheLevel: ASTNode.CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description =
    "Increments a counter stat eventwise: botmaker/<app>/botstats/<name1>/<name2>"
  override def arguments: Seq[String] = Seq[String]("stat names, e.g., [name1, name2]")
  override def examples: Seq[String] = Seq[String]("""IncrementStatsEventwise(["Test", "Run"])""")

  private[this] val cachedException = new RuntimeFailure(
    "IncrementStatEventwise needs an EventContext.")

  override def evaluate(context: Context[TwitterRuntime], statNames: List[String]): Unit = {
    if (statNames.forall(CharMatcher.ascii.matchesAllOf)) {
      context.getRootContext match {
        case rc: RuleContext[_] =>
          val names = BotStats.Prefix :: statNames
          rc.eventContext.cacheGet(this, names) {
            context.getRuntime.statsReceiver.counter(names: _*).incr()
          }
        case _ =>
          throw cachedException
      }
    }
  }
}
