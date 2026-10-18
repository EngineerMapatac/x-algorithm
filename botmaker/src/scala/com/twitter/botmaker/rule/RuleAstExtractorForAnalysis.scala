package com.twitter.botmaker.rule

import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerDoc
import com.twitter.botmaker.compiler.Fingerprint
import com.twitter.botmaker.function.DerivedFeatureCall
import com.twitter.botmaker.rule.DefaultRuleAstExtractorForAnalysis._
import com.twitter.relevance.feature_store.thrift.FeatureData
import com.twitter.relevance.feature_store.thrift.FeatureValue
import scala.collection.JavaConverters._
import scala.collection.mutable

trait RuleAstExtractorForAnalysis {

  def analyzeRule(ruleId: Long): Option[Map[String, FeatureData]]
}

class DefaultRuleAstExtractorForAnalysis(
  private[this] val ruleContainer: RuleContainer[_])
    extends RuleAstExtractorForAnalysis {

  private[this] val botMakerDocsMap: Map[String, BotMakerDoc] =
    ruleContainer.compiler.getBotMakerDocs.asScala
      .groupBy(_.name).transform {
        case (_, docsMatching) => docsMatching.head
      }

  override def analyzeRule(ruleId: Long): Option[Map[String, FeatureData]] = {
    ruleContainer
      .getRule(ruleId).map(rule => {
        val visited = mutable.Set[Fingerprint]()
        val output = mutable.Map[String, mutable.Set[String]]()
        indexForEffects(rule.booleanAstRoot, visited, output)
        indexForEffects(rule.actionAstRoot, visited, output)
        output.mapValues { outputSet =>
          val featureValue = new FeatureValue()
          featureValue.setStrSetValue(outputSet.asJava)
          new FeatureData().setFeatureValue(featureValue)
        }.toMap
      })
  }

  private def indexForEffects(
    node: ASTNode[_],
    visited: mutable.Set[Fingerprint],
    output: mutable.Map[String, mutable.Set[String]]
  ): Unit = {
    if (visited.add(node.getFingerprint)) {
      node match {
        case _: DerivedFeatureCall =>
        case func =>
          indexFunctionIfAppropriate(func, output)
      }
      node.getChildren.forEach(child => indexForEffects(child, visited, output))
    }
  }

  private def indexFunctionIfAppropriate(
    node: ASTNode[_],
    output: mutable.Map[String, mutable.Set[String]]
  ): Unit = {
    val functionName = node.getClassName
    botMakerDocsMap.get(functionName) match {
      case Some(functionDoc) =>
        val mapKeyOpt = functionDoc.actionLevel match {
          case ActionLevel.PROD_TANGIBLE_IMPACT_ACTION => Some(ProdTangibleImpact)
          case ActionLevel.PROD_AGENT_WORKFLOW_ACTION => Some(ProdAgentWorkflow)
          case ActionLevel.PROD_OTHER_ACTION => Some(ProdOther)
          case ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION => Some(ResponseCodeChange)
          case ActionLevel.RESPONSE_CODE_NO_CHANGE_ACTION => Some(ResponseCodeNoChange)
          case _ => None
        }
        mapKeyOpt match {
          case Some(mapKey) =>
            output.getOrElseUpdate(mapKey, mutable.Set()).add(functionName)
          case _ =>
        }
      case _ =>
    }
  }

}

object DefaultRuleAstExtractorForAnalysis {
  final val ProdTangibleImpact = "prodTangibleImpact"
  final val ProdAgentWorkflow = "prodAgentWorkflow"
  final val ProdOther = "prodOther"
  final val ResponseCodeChange = "responseCodeChange"
  final val ResponseCodeNoChange = "responseCodeNoChange"
}
