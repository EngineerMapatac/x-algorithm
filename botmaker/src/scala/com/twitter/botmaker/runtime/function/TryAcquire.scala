package com.twitter.botmaker.runtime.function

import com.twitter.botmaker.FunctionUnit1
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.exceptions.RuntimeFailure
import com.twitter.botmaker.rule.RuleContext
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.DownstreamService

class TryAcquire extends FunctionUnit1[TwitterRuntime, String, Boolean] {

  override def cacheLevel: ASTNode.CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Never
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description = "Tries to acquire an event-scope semaphore defined by :name."
  override def arguments: Seq[String] = Seq[String](":name")

  override def evaluate(
    context: com.twitter.botmaker.Context[TwitterRuntime],
    name: String
  ): Boolean = {
    context.getRootContext match {
      case rc: RuleContext[_] =>
        val cached = new Object
        cached == rc.eventContext.cacheGet(this, name) {
          cached
        }
      case _ =>
        throw new RuntimeFailure("TryAcquire needs to run in RuleContext.")
    }
  }
}

object TryAcquire extends TryAcquire
