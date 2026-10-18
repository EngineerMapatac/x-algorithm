package com.twitter.botmaker.rule

import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.rule.thriftscala.ActionLimit
import com.twitter.util.Future

trait ActionRateLimiter[RT] {
  def applyActionRateLimit[V](
    context: Context[RT],
    functionName: String,
    actionLevel: ActionLevel,
    ruleId: Long,
    actionLimits: collection.Map[String, Seq[ActionLimit]],
    func: () => Future[V],
    args: Seq[_]
  ): Future[V]
}

object ActionRateLimiter {
  def empty[RT]: ActionRateLimiter[RT] = new ActionRateLimiter[RT] {
    override def applyActionRateLimit[V](
      context: Context[RT],
      functionName: String,
      actionLevel: ActionLevel,
      ruleId: Long,
      actionLimits: collection.Map[String, Seq[ActionLimit]],
      func: () => Future[V],
      args: Seq[_]
    ): Future[V] = func.apply()
  }
}
