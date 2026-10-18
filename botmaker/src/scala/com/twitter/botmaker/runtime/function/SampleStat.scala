package com.twitter.botmaker.runtime.function

import com.google.common.base.CharMatcher
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit2

object SampleStat extends FunctionUnit2[TwitterRuntime, String, Long, Unit] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Never
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def description: String =
    "Adds a percentile stat sample: botmaker/<app>/botstats/statName"

  override def arguments: Seq[String] = Seq("stat name", "sample value")

  override def evaluate(context: Context[TwitterRuntime], statName: String, value: Long): Unit = {
    if (CharMatcher.ascii.matchesAllOf(statName)) {
      context.getRuntime.statsReceiver.stat(BotStats.Prefix, statName).add(value)
    }
  }

}
