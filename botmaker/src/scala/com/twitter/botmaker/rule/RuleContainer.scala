package com.twitter.botmaker.rule

import java.util.Collections
import java.util.{HashMap => JHashMap}
import java.util.{IdentityHashMap => JIdentityHashMap}
import java.util.{List => JList}
import java.util.{Map => JMap}
import java.util.{Set => JSet}
import scala.collection.JavaConverters._
import com.google.common.collect.ImmutableList
import com.google.common.collect.ImmutableMap
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Extractor
import com.twitter.botmaker.compiler.Compiler
import com.twitter.botmaker.compiler.CompilerContext
import com.twitter.botmaker.compiler.DerivedFeatureProvider
import com.twitter.botmaker.compiler.Fingerprint
import com.twitter.botmaker.compiler.UsageUnit
import com.twitter.botmaker.compiler.exceptions.ParseFailure
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.tuples.Pair
import com.twitter.botmaker.rule.RuleContainer.Node
import com.twitter.botmaker.rule.thriftscala.BotMakerRule
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.finagle.stats.NullStatsReceiver
import com.twitter.finagle.stats.StatsReceiver
import javax.annotation.concurrent.NotThreadSafe
import org.slf4j.Logger
import org.slf4j.LoggerFactory

class RuleContainer[T <: TwitterRuntime] private (
  val compiler: Compiler[T],
  private val nodes: Map[Long, Node[T]],
  private val _evaluators: Map[String, EventEvaluator[T]]) {

  val numRules: Int = nodes.size

  val numEvents: Int = _evaluators.keys.size

  val numMappedRules: Int = _evaluators.values.map(_.ruleEvaluators.size).sum

  val failedRules: Seq[FakeEvaluator[_]] = {
    nodes.values.toSeq.map(_.ruleEvaluator) collect {
      case r: FakeEvaluator[_] => r
    }
  }

  val hasCompilingFailure: Boolean = {
    nodes.values.map(_.ruleEvaluator).exists(_.hasCompilingFailure)
  }

  def isEventDefined(eventType: String): Boolean = {
    _evaluators.contains(eventType)
  }

  def getRule(id: Long): Option[RuleAst] = {
    nodes.get(id).map(_.ruleEvaluator)
  }

  def rules: Set[Long] = nodes.keySet

  def getEventEvaluator(eventType: String): EventEvaluator[T] = {
    _evaluators(eventType)
  }

  def evaluators: Map[String, EventEvaluator[T]] = this._evaluators
}

object RuleContainer {
  private val logger: Logger = LoggerFactory.getLogger(getClass)
  val DefaultExtractorVisitorFactory: () => CacheableExtractorSetter =
    () => new CacheableExtractorSetter

  private case class Node[T <: TwitterRuntime](
    ruleEvaluator: RuleEvaluator[T],
    usageTracker: Set[UsageUnit]) {

    def ruleId: Long = ruleEvaluator.botMakerRule.id

    def eventType: String = ruleEvaluator.botMakerRule.eventType

    def isReusable(
      compilerContext: CompilerContext[T],
      derivedFeatureProvider: DerivedFeatureProvider,
      botMakerRule: BotMakerRule
    ): Boolean = {
      if (ruleEvaluator.hasCompilingFailure) {
        false
      } else {
        val cachedRule: BotMakerRule = ruleEvaluator.botMakerRule
        val isEquivalent: Boolean = areEquivalent(cachedRule, botMakerRule)
        if (!isEquivalent) {
          logger.info(s"cached bot $cachedRule is not equivalent to $botMakerRule")
        }
        isEquivalent && {
          usageTracker.forall { usageUnit: UsageUnit =>
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
              logger.warn("encountered an unknown usage unit. default to recompiling the rule")
              false
            }
          }
        }
      }
    }
  }

  private def areEquivalent(rule1: BotMakerRule, rule2: BotMakerRule): Boolean = {
    rule1.id == rule2.id &&
    rule1.name == rule2.name &&
    rule1.eventType == rule2.eventType &&
    rule1.actionExpression.replaceAll("\\n\\s+", "\n") == rule2.actionExpression
      .replaceAll("\\n\\s+", "\n") &&
    rule1.booleanExpression.replaceAll("\\n\\s+", "\n") == rule2.booleanExpression
      .replaceAll("\\n\\s+", "\n") &&
    rule1.actionLimits.equals(rule2.actionLimits)
  }

  def apply[T <: TwitterRuntime](compiler: Compiler[T]): RuleContainer[T] = {
    new RuleContainer[T](compiler, Map.empty, Map.empty)
  }

  def apply[T <: TwitterRuntime](
    container: RuleContainer[T],
    compiler: Compiler[T],
    botMakerRules: Seq[BotMakerRule],
    derivedFeatures: DerivedFeatureProvider,
    exceptionClassifier: Throwable => ExceptionCategory,
    enableContextScope: BotMakerRule => Boolean = _ => true,
    eventMappings: JMap[String, JList[String]] = ImmutableMap.of(),
    statsReceiver: StatsReceiver = NullStatsReceiver
  ): RuleContainer[T] = {
    val compilerContext = compiler
      .createContextBuilder()
      .setDerivedFeatures(derivedFeatures)
      .build()
    val derivedFeatureCache = new JHashMap[String, Pair[ASTNode[T], CompilerContext[T]]]
    val compiledRules = botMakerRules.map { botMakerRule =>
      if (container.nodes.contains(botMakerRule.id)
        && container
          .nodes(botMakerRule.id).isReusable(
            compilerContext = compilerContext,
            derivedFeatureProvider = derivedFeatures,
            botMakerRule = botMakerRule)) {
        statsReceiver.counter("compile", "skipped").incr()
        container.nodes(botMakerRule.id)
      } else {
        if (!container.nodes.contains(botMakerRule.id)) {
          logger.info(s"bot ${botMakerRule.id} not cached")
        }
        compile(
          compiler = compiler,
          statsReceiver = statsReceiver,
          botMakerRule = botMakerRule,
          derivedFeatures = derivedFeatures,
          derivedFeatureCache = derivedFeatureCache,
          enableContextScope = enableContextScope,
          exceptionClassifier = exceptionClassifier
        )
      }
    }

    val evaluators = compiledRules.flatMap { node =>
      val eventTypes = eventMappings.getOrDefault(node.eventType, ImmutableList.of(node.eventType))
      eventTypes.asScala.map { e => e -> node.ruleEvaluator }
    } groupBy {
      case (eventType, _) => eventType
    } map {
      case (eventType, seq) => eventType -> EventEvaluator[T](eventType, seq.map(_._2))
    }

    traverseExtractors(evaluators, DefaultExtractorVisitorFactory())

    new RuleContainer[T](
      compiler = compiler,
      nodes = compiledRules.map(node => node.ruleId -> node).toMap,
      _evaluators = evaluators
    )
  }

  private def compile[T <: TwitterRuntime](
    compiler: Compiler[T],
    statsReceiver: StatsReceiver,
    botMakerRule: BotMakerRule,
    derivedFeatures: DerivedFeatureProvider,
    derivedFeatureCache: JMap[String, Pair[ASTNode[T], CompilerContext[T]]],
    enableContextScope: BotMakerRule => Boolean,
    exceptionClassifier: Throwable => ExceptionCategory
  ): Node[T] = {

    logger.info(
      s"compiling bot ${botMakerRule.id} - ${botMakerRule.name} - " +
        s"enableContextScope=${enableContextScope(botMakerRule)}")

    val stat: StatsReceiver = statsReceiver.scope("stats")

    val inputFeatureNameSpace: String = botMakerRule.eventType

    val context: CompilerContext[T] = compiler
      .createContextBuilder()
      .setDerivedFeatures(derivedFeatures)
      .setInputFeatureNamespace(inputFeatureNameSpace)
      .setDerivedFeatureCache(derivedFeatureCache)
      .enableContextScope(enableContextScope(botMakerRule))
      .build()

    try {
      val booleanAst: ASTNode[T] = context.compile(safe(botMakerRule.booleanExpression))
      if (Type.isDivergentTo(Type.BOOLEAN, booleanAst.getReturnType)) {
        throw new SemanticCheckFailure(
          "Boolean expression expects return type of %s but %s received.".format(
            Type.BOOLEAN,
            booleanAst.getReturnType
          )
        )
      }
      val actionAst: ASTNode[T] = context.compile(safe(botMakerRule.actionExpression))
      val ruleEvaluator = new CompiledEvaluator[T](
        botMakerRule,
        booleanAst.toExtractor,
        actionAst.toExtractor,
        stat,
        exceptionClassifier
      )
      logger.info(s"finished compiling bot ${botMakerRule.id} - ${botMakerRule.name}")
      statsReceiver.counter("compile", "successes").incr()
      Node(ruleEvaluator, context.getUsageTracker.asScala.toSet)
    } catch {
      case parseException: ParseFailure =>
        logger.warn(
          "Parse failure on bot: %d, %s".format(botMakerRule.id, parseException)
        )
        statsReceiver.counter("compile", "parse_failures").incr()
        val ruleEvaluator = new FakeEvaluator[T](
          botMakerRule,
          parseException,
          stat
        )
        Node(ruleEvaluator, context.getUsageTracker.asScala.toSet)
      case semanticCheckFailure: SemanticCheckFailure =>
        logger.warn(
          "Semantic check failure on bot: %d, %s".format(botMakerRule.id, semanticCheckFailure)
        )
        statsReceiver.counter("compile", "semantic_check_failures").incr()
        val ruleEvaluator = new FakeEvaluator[T](
          botMakerRule,
          semanticCheckFailure,
          stat
        )
        Node(ruleEvaluator, context.getUsageTracker.asScala.toSet)
      case e =>
        logger.warn(s"unexpected failure on bot: ${botMakerRule.id} ($e)", e)
        statsReceiver.counter("compile", "unexpected_failures").incr()
        val ruleEvaluator = new FakeEvaluator[T](
          botMakerRule,
          e,
          stat
        )
        Node(ruleEvaluator, context.getUsageTracker.asScala.toSet)
    }
  }

  private[this] val skipCaches =
    Set("Constant", "Variable", "InputFeature", "Val", "Exists", "CacheGet")

  def traverseExtractors[T <: TwitterRuntime](
    eventEvaluators: Map[String, EventEvaluator[T]],
    extractorVisitor: ExtractorVisitorUnit
  ): Unit = {
    for (eventEvaluator <- eventEvaluators.values;
      evaluator <- eventEvaluator.ruleEvaluators) {
      evaluator match {
        case x: CompiledEvaluator[T] =>
          extractorVisitor.onRuleInit(x.botMakerRule)
          x.booleanExtractor.acceptDefinedVisitor(extractorVisitor.visitor)
          x.actionExtractor.acceptDefinedVisitor(extractorVisitor.visitor)
          extractorVisitor.onRuleExit()
        case _ => ()
      }
    }
    extractorVisitor.onCompletion()
  }

  @NotThreadSafe
  class CacheableExtractorSetter extends ExtractorVisitorUnit {
    private var totalCount = 0L
    private var globalCount = 0L
    private var eventCount = 0L
    private var constantCount = 0L
    private var nocacheCount = 0L
    private val visited: JSet[Extractor[_]] =
      Collections.newSetFromMap(new JIdentityHashMap)
    private val _visitor: PartialFunction[Extractor[_], Unit] =
      new PartialFunction[Extractor[_], Unit] {
        override def apply(extractor: Extractor[_]): Unit = {
          totalCount += 1
          visited.add(extractor)
          val fingerprint = extractor.getFingerprint
          val scope = extractor.getFingerprint.scope
          if (fingerprint == Fingerprint.EMPTY) {
            nocacheCount += 1
            extractor.setCacheable(false)
          } else if (skipCaches.contains(extractor.getClassName)) {
            constantCount += 1
            extractor.setCacheable(false)
          } else if (scope == ASTNode.CacheLevel.Global.scope) {
            globalCount += 1
            extractor.setCacheable(true)
          } else if (scope == ASTNode.CacheLevel.Event.scope) {
            eventCount += 1
            extractor.setCacheable(true)
          } else {
            nocacheCount += 1
            extractor.setCacheable(false)
          }
        }

        override def isDefinedAt(extractor: Extractor[_]): Boolean = !visited.contains(extractor)
      }

    override def visitor: PartialFunction[Extractor[_], Unit] = _visitor

    override def onRuleInit(rule: BotMakerRule): Unit = ()

    override def onRuleExit(): Unit = ()

    override def onCompletion(): Unit =
      logger.info(
        "Fingerprints scope: total(%d), global(%d), constant(%d), event(%d), none(%d)".format(
          totalCount,
          globalCount,
          constantCount,
          eventCount,
          nocacheCount
        )
      )
  }
}
