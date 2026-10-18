package com.twitter.botmaker.rule.function

import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit2
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.rule.BotMaker
import com.twitter.botmaker.rule.UsageInfo
import com.twitter.botmaker.rule.thriftscala.BotMakerRulesPackage
import com.twitter.botmaker.runtime.TwitterRuntime

object GetFunctionOrFeatureDirectUsages
    extends FunctionUnit2[TwitterRuntime, BotMakerRulesPackage, String, Map[String, Set[String]]] {
  override def cacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description: String =
    """get the direct references to a named input feature, function or derived feature in a
      |rules package, returns a map with two keys -- "rules" and "derived features", with values
      |being sets of rule IDs and derived feature names directly using the input feature, function
      |or derived feature
    """.stripMargin
  override def arguments: Seq[String] =
    Seq("rules package", "name of input feature, function or derived feature")

  override def evaluate(
    context: Context[TwitterRuntime],
    rulesPackage: BotMakerRulesPackage,
    functionOrFeatureName: String
  ): Map[String, Set[String]] = {
    val usageInfo: UsageInfo =
      BotMaker.getFunctionOrFeatureDirectUsages(rulesPackage, functionOrFeatureName)
    Map(
      "rules" -> usageInfo.ruleIds.map(_.toString),
      "derived features" -> usageInfo.derivedFeatureNames
    )
  }
}
