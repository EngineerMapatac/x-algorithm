package com.twitter.botmaker.rule

import com.twitter.botmaker.compiler.ActionLevel

trait ProfilingCollector {
  def reportFunctionEvaluation(
    functionName: String,
    actionLevel: ActionLevel,
    args: Seq[_],
    ruleId: Long
  ): Unit
}

object ProfilingCollector {
  val empty: ProfilingCollector = new ProfilingCollector {
    override def reportFunctionEvaluation(
      functionName: String,
      actionLevel: ActionLevel,
      args: Seq[_],
      ruleId: Long
    ): Unit =
      ()
  }
}
