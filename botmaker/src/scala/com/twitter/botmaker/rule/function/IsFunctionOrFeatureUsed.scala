package com.twitter.botmaker.rule.function

import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit2
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.rule.BotMaker
import com.twitter.botmaker.rule.thriftscala.BotMakerRulesPackage
import com.twitter.botmaker.runtime.TwitterRuntime

object IsFunctionOrFeatureUsed
    extends FunctionUnit2[TwitterRuntime, BotMakerRulesPackage, String, Boolean] {
  override def cacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description: String =
    "Check if a named input feature, function or derived feature is used in a rules package"
  override def arguments: Seq[String] =
    Seq("rules package", "name of input feature, function or derived feature")

  override def evaluate(
    context: Context[TwitterRuntime],
    rulesPackage: BotMakerRulesPackage,
    functionOrFeatureName: String
  ): Boolean = {
    BotMaker.isFunctionOrFeatureUsed(rulesPackage, functionOrFeatureName)
  }
}
