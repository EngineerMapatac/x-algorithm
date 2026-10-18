package com.twitter.botmaker.runtime.function

import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit2
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.tuples.Pair
import com.twitter.botmaker.runtime.TwitterRuntime

object BotStats {
  val Prefix: String = "botstats"
}

object IncrementStat extends FunctionUnit2[TwitterRuntime, String, Long, Unit] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Never
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description: String = "Increments a counter stat: botmaker/<app>/botstats/statName"
  override def arguments: Seq[String] = Seq("statName", "amount to increment by")

  val invalidStatChars = """[^a-zA-Z0-9._\-:/]+""".r

  override def evaluate(context: Context[TwitterRuntime], statName: String, value: Long): Unit = {
    var replacementMade = false
    val sanitizedStatName = invalidStatChars.replaceAllIn(
      statName,
      _ => {
        replacementMade = true
        "_"
      })
    if (replacementMade) {
      context.getRuntime.logger.info(
        Pair.of(this, context.getRuntime.name),
        s"go/bot/${context.getRuntime.name}/${context.getTrackingId} IncrementStat: invalid stat supplied=${statName} updated=${sanitizedStatName}"
      )
    }
    context.getRuntime.statsReceiver.counter(BotStats.Prefix, sanitizedStatName).incr(value)

  }
}
