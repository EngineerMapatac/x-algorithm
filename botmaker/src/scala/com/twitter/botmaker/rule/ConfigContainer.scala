package com.twitter.botmaker.rule

import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.Extractor
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.Compiler
import com.twitter.botmaker.compiler.CompilerContext
import com.twitter.botmaker.compiler.DerivedFeatureProvider
import com.twitter.botmaker.compiler.Fingerprint
import com.twitter.botmaker.compiler.UsageUnit
import com.twitter.botmaker.compiler.tuples.StackFrame
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.rule.ConfigContainer.Node
import com.twitter.botmaker.rule.thriftscala.BotMakerRule
import com.twitter.botmaker.runtime.config.ServiceConfigs
import com.twitter.botmaker.util.TriConsumer
import com.twitter.finagle.stats.NullStatsReceiver
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util.Await
import com.twitter.util.Duration
import com.twitter.util.Future
import java.util.function.BiFunction
import java.util.function.Supplier
import java.util.logging.Logger
import java.util.{List => JList}
import scala.collection.JavaConverters._

class ConfigContainer[T <: ServiceConfigs] private (
  val compiler: Compiler[T],
  val systemConfigs: T,
  private val inputFeatures: Map[String, AnyRef],
  private val nodes: Map[Long, Node[T]]) {}

object ConfigContainer {

  private class Node[T <: ServiceConfigs](
    botmakerRule: BotMakerRule,
    booleanAst: ASTNode[T],
    actionAst: ASTNode[T],
    usageTracker: Set[UsageUnit])
      extends RuleAst(botmakerRule, booleanAst, actionAst) {

    val booleanExtractor: Extractor[T] = booleanAst.toExtractor
    val actionExtractor: Extractor[T] = actionAst.toExtractor

    def isReusable(
      compiler: Compiler[T],
      compilerContext: CompilerContext[T],
      rule: BotMakerRule,
      derivedFeatureProvider: DerivedFeatureProvider
    ): Boolean = {
      botmakerRule == rule && usageTracker.forall { usageUnit: UsageUnit =>
        if (usageUnit.hasDerivedFeature) {
          val derivedFeatureFingerprint: Fingerprint =
            usageUnit.getDerivedFeatureFingerprint.get()
          compilerContext.containsFingerprint(derivedFeatureFingerprint)
        } else if (usageUnit.hasCompilerUnit) {
          val compilerUnitFingerprint: Fingerprint =
            usageUnit.getCompilerUnitFingerprint.get()
          compilerContext.containsFingerprint(compilerUnitFingerprint)
        } else if (usageUnit.hasInputFeature) {
          val inputFeatureName: String = usageUnit.getInputFeatureName.get()
          !derivedFeatureProvider.get(inputFeatureName).isPresent
        } else {
          logger.warning("encountered an unknown usage unit. default to recompiling the rule.")
          false
        }
      }
    }
  }

  private val logger: Logger = Logger.getLogger(getClass.getName)

  def apply[T <: ServiceConfigs](
    compiler: Compiler[T],
    inputFeatures: Map[String, AnyRef]
  )(
    implicit manifest: Manifest[T]
  ): ConfigContainer[T] = {
    val configs = manifest.runtimeClass.newInstance.asInstanceOf[T]
    new ConfigContainer[T](compiler, configs, inputFeatures, Map.empty)
  }

  def apply[T <: ServiceConfigs](
    container: ConfigContainer[T],
    botmakerRules: Seq[BotMakerRule],
    derivedFeatures: DerivedFeatureProvider,
    statsReceiver: StatsReceiver = NullStatsReceiver,
    timeout: Duration = Duration.fromSeconds(5)
  )(
    implicit manifest: Manifest[T]
  ): ConfigContainer[T] = {

    val configs = manifest.runtimeClass.newInstance.asInstanceOf[T]
    val compiler = container.compiler
    val inputFeatures = container.inputFeatures

    val compilerContext = compiler
      .createContextBuilder()
      .setDerivedFeatures(derivedFeatures)
      .build()
    val compiledRules = botmakerRules.map { botmakerRule =>
      if (container.nodes.contains(botmakerRule.id) &&
        container
          .nodes(botmakerRule.id).isReusable(
            compiler,
            compilerContext,
            botmakerRule,
            derivedFeatures)) {
        statsReceiver.counter("compile", "skipped").incr()
        container.nodes(botmakerRule.id)
      } else {
        compile(compiler, botmakerRule, derivedFeatures, statsReceiver)
      }
    }

    Await.result(
      Future.collect {
        compiledRules.map {
          node =>
            val context = createContext(configs, node.id, inputFeatures)
            context.run(node.booleanExtractor) flatMap {
              case condition: java.lang.Boolean if condition =>
                context.run(node.actionExtractor).unit
              case _ =>
                Future.Unit
            } onFailure { t =>
              logger.warning(s"Failed to execute config rule ${node.name}: $t")
              statsReceiver.counter("failures").incr()
            }
        }
      },
      timeout
    )

    new ConfigContainer(
      compiler,
      configs,
      inputFeatures,
      compiledRules.map(node => node.id -> node).toMap)
  }

  private def createContext[T <: ServiceConfigs](
    configs: T,
    ruleId: Long,
    inputFeatures: Map[String, AnyRef]
  ) = {

    val startTimeMillis = System.currentTimeMillis

    new Context[T] {
      override val runtimeSupplier: Supplier[T] = () => configs
      override def getTrackingId: Long = ruleId
      override def nowMillis: Long = System.currentTimeMillis
      override def startMillis: Long = startTimeMillis

      override protected def extractorEvaluationFunction: BiFunction[Context[T], Extractor[
        T
      ], Future[AnyRef]] =
        (ctx, extractor) => extractor.run(ctx)

      override def inputFeatureFunction: PartialFunction[String, AnyRef] = inputFeatures
      override def addOutputFeature(ns: scala.Any, n: String, t: Type, v: scala.Any): Unit = {}
      override def reportStat(ns: scala.Any, stat: String, v: Long): Unit = {}
      override def reportFailure(ns: scala.Any, cat: String, t: Throwable): Unit = {}
      override def getStackFrames: JList[StackFrame] = Context.DEFAULT_STACK_FRAMES.get()
      override def reportFunctionEvaluationConsumer: TriConsumer[String, ActionLevel, JList[_]] = {
        (_, _, _) =>
      }
      override def applyActionRateLimit[V](
        context: Context[T],
        functionName: String,
        actionLevel: ActionLevel,
        func: () => Future[V],
        args: JList[_]
      ): Future[V] = func.apply()
    }
  }

  private def compile[T <: ServiceConfigs](
    compiler: Compiler[T],
    botmakerRule: BotMakerRule,
    derivedFeatures: DerivedFeatureProvider,
    statsReceiver: StatsReceiver
  ): Node[T] = {
    try {
      val inputFeatureNameSpace: String = botmakerRule.eventType

      val context = compiler
        .createContextBuilder()
        .setDerivedFeatures(derivedFeatures)
        .setInputFeatureNamespace(inputFeatureNameSpace)
        .build()
      val conditionAst = context.compile(botmakerRule.booleanExpression)
      val actionAst = context.compile(botmakerRule.actionExpression)
      statsReceiver.counter("compile", "successes").incr()
      new Node[T](botmakerRule, conditionAst, actionAst, context.getUsageTracker.asScala.toSet)
    } catch {
      case t: Throwable =>
        logger.warning(s"Failed to parse config rule ${botmakerRule.name}: $t")
        statsReceiver.counter("compile", "failures").incr()
        throw t
    }
  }
}
