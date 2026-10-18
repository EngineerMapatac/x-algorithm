package com.twitter.botmaker.runtime.function

import com.google.common.base.CharMatcher
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit2

object IncrementStats extends FunctionUnit2[TwitterRuntime, List[String], Long, Unit] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Never
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description =
    "Increments a counter stat: botmaker/<app>/botstats/<name1>/<name1>/..."
  override def arguments: Seq[String] = Seq(
    "stat names, e.g., [name1, name2]",
    "amount to increment by"
  )

  override def evaluate(
    context: Context[TwitterRuntime],
    statNames: List[String],
    value: Long
  ): Unit = {
    if (statNames.forall(CharMatcher.ascii.matchesAllOf)) {
      val names = BotStats.Prefix :: statNames
      context.getRuntime.statsReceiver.counter(names: _*).incr(value)
    }
  }
}
