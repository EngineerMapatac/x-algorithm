package com.twitter.botmaker.util

import com.twitter.botmaker.antlr.BotMakerLexer
import com.twitter.botmaker.compiler.Parser
import com.twitter.botmaker.rule.thriftscala.BotMakerDerivedFeature
import com.twitter.botmaker.rule.thriftscala.BotMakerRule
import com.twitter.botmaker.rule.thriftscala.BotMakerRulesPackage
import org.antlr.runtime.tree.Tree
import scala.collection.mutable

object CodeAnalysisUtil {
  def filterUsedDfs(pkg: BotMakerRulesPackage): BotMakerRulesPackage = {
    val directDfCallIndex: Map[String, Set[String]] = indexDirectDfCalls(rulesPackage = pkg)
    val usedDfNames: Set[String] = pkg.rules.toSet.flatMap { bot: BotMakerRule =>
      val conditionExprTree: Tree = Parser.createParseTree(bot.booleanExpression)
      val actionExprTree: Tree = Parser.createParseTree(bot.actionExpression)
      getNamesOfCalledDfFns(
        exprTree = conditionExprTree,
        directDfCallIndex = directDfCallIndex) ++ getNamesOfCalledDfFns(
        exprTree = actionExprTree,
        directDfCallIndex = directDfCallIndex)
    }
    pkg.copy(derivedFeatures = pkg.derivedFeatures.filter { df: BotMakerDerivedFeature =>
      usedDfNames.contains(df.name)
    })
  }

  private def indexDirectDfCalls(
    rulesPackage: BotMakerRulesPackage
  ): Map[String, Set[String]] = {
    rulesPackage.derivedFeatures.map { df: BotMakerDerivedFeature =>
      df.name -> getNamesOfDirectlyCalledDfFns(
        Parser.createParseParamsTree(df.featureExpression).getFirst)
    }.toMap
  }

  private def getNamesOfCalledDfFns(
    exprTree: Tree,
    directDfCallIndex: Map[String, Set[String]]
  ): Set[String] = {
    val directlyCalled: Set[String] = getNamesOfDirectlyCalledDfFns(exprTree = exprTree)
    getTransitiveClosure(start = directlyCalled, index = directDfCallIndex)
  }

  private def getNamesOfDirectlyCalledDfFns(exprTree: Tree): Set[String] = {
    val rootDfFnCall: Set[String] = exprTree.getType match {
      case BotMakerLexer.FEATURE | BotMakerLexer.FUNCTION => Set(exprTree.getText)
      case _ => Set.empty
    }
    val dfFnCallsInChildren: Set[String] = (0 until exprTree.getChildCount).flatMap { i: Int =>
      getNamesOfDirectlyCalledDfFns(exprTree = exprTree.getChild(i))
    }.toSet

    rootDfFnCall ++ dfFnCallsInChildren
  }

  private def getTransitiveClosure(
    start: Set[String],
    index: Map[String, Set[String]]
  ): Set[String] = {
    val visited: mutable.Set[String] = mutable.Set.empty
    val closure: mutable.Set[String] = mutable.Set.empty
    val queue: mutable.Queue[String] = mutable.Queue.empty
    queue ++= start
    while (queue.nonEmpty) {
      val s: String = queue.dequeue()
      if (!visited.contains(s)) {
        visited += s
        if (index.contains(s)) {
          closure += s
          closure ++= index(s).filter(index.contains)
          queue ++= index(s).filter { item => !visited.contains(item) && index.contains(item) }
        }
      }
    }
    closure.toSet
  }
}
