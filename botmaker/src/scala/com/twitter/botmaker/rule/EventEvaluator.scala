package com.twitter.botmaker.rule

import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.util.Duration
import com.twitter.util.Future

case class EventEvaluator[T <: TwitterRuntime](
  eventType: String,
  ruleEvaluators: Seq[RuleEvaluator[T]]) {

  def evaluate(eventContext: EventContext[T], timeout: Duration): Future[Unit] = {

    val results = ruleEvaluators map { ruleEvaluator =>
      if (eventContext.skipRules.contains(ruleEvaluator.botMakerRule.id)) {
        eventContext.addOutputFeature(
          RuleResultOutput,
          ruleEvaluator.botMakerRule.id,
          RuleResult.SKIPPED.toString,
          Type.OBJECT,
          RuleResult.SKIPPED
        )
        Future.Unit
      } else {
        val ruleContext = RuleContext(eventContext, ruleEvaluator.botMakerRule)
        ruleEvaluator.evaluate(ruleContext, timeout)
      }
    }

    Future.join(results)
  }
}
