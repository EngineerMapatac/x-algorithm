package com.twitter.botmaker.rule.function

import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit1
import com.twitter.botmaker.rule.BotMakerRuntime
import com.twitter.botmaker.rule.thriftscala.BotMakerRule

object GetRule extends FunctionUnit1[BotMakerRuntime[_], Long, BotMakerRule] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description: String = "Get BotMaker rule."
  override def arguments: Seq[String] = Nil

  override def evaluate(context: Context[BotMakerRuntime[_]], ruleId: Long): BotMakerRule = {
    val ruleAstOpt = context.getRuntime.botmaker.state.ruleContainer.getRule(ruleId).orElse {
      context.getRuntime.botmaker.state.systemContainer.getRule(ruleId)
    }
    ruleAstOpt
      .map(_.botMakerRule).getOrElse(throw new IllegalArgumentException(s"No rule $ruleId found"))
  }
}
