package com.twitter.botmaker.rule

import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.Extractor
import com.twitter.botmaker.Extractor.Functor
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.exceptions.ContextCancelledException
import com.twitter.botmaker.compiler.exceptions.RuntimeFailure
import com.twitter.botmaker.compiler.tuples.StackFrame
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.rule.thriftscala.ActionLimit
import com.twitter.botmaker.rule.thriftscala.BotMakerRule
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.util.TriConsumer
import com.twitter.util.Future
import java.util.function.BiFunction
import java.util.function.Supplier
import java.util.function.{BooleanSupplier => JBooleanSupplier}
import java.util.{List => JList}
import scala.collection.JavaConverters._

object RuleContext {

  def apply[T <: TwitterRuntime](
    eventContext: EventContext[T],
    rule: BotMakerRule
  ): RuleContext[T] = {
    new RuleContext(eventContext, rule, eventContext.startMillis)
  }

  def apply[T <: TwitterRuntime](
    eventContext: EventContext[T],
    rule: BotMakerRule,
    startMillis: Long
  ): RuleContext[T] = {
    new RuleContext(eventContext, rule, startMillis)
  }

  def apply[T <: TwitterRuntime](
    eventContext: EventContext[T],
    ruleId: Long,
    eventType: String = "",
    name: String = "",
    expr: String = "TRUE"
  ): RuleContext[T] = {
    val rule = BotMakerRule(
      id = ruleId,
      name = name,
      eventType = eventType,
      booleanExpression = "TRUE",
      actionExpression = expr)
    new RuleContext(eventContext, rule, eventContext.startMillis)
  }
}

class RuleContext[T <: TwitterRuntime](
  val eventContext: EventContext[T],
  val rule: BotMakerRule,
  override val startMillis: Long)
    extends Context[T] {

  @volatile private var ruleResult: RuleResult.Value = RuleResult.NONE

  @volatile private var cancelled: Boolean = false

  def result: RuleResult.Value = ruleResult

  def result_=(result: RuleResult.Value): Unit = {
    if (ruleResult != RuleResult.CANCELLED) {
      ruleResult = result
    }
  }

  override val runtimeSupplier: Supplier[T] = () => eventContext.runtime

  def eventType: String = eventContext.eventType

  override def getTrackingId: Long = rule.id

  override def nowMillis: Long = getRuntime.clock.nowMillis

  override val cancelledSupplier: JBooleanSupplier = () => cancelled

  def cancel(): Unit = {
    ruleResult = RuleResult.CANCELLED
    cancelled = true
  }

  override protected val extractorEvaluationFunction: BiFunction[Context[T], Extractor[T], Future[
    AnyRef
  ]] = (ctx, extractor) => {
    if (ctx.isCancelled) {
      Future.exception(new ContextCancelledException)
    } else if (ctx.isDebugging) {
      extractor.run(ctx)
    } else if (extractor.isCacheable &&
      extractor.getFingerprint.scope == ASTNode.CacheLevel.Global.scope) {
      getRuntime.cacheGet(extractor.getFingerprint) {
        extractor.run(ctx)
      } match {
        case f: Future[AnyRef] => f
        case _ => Future.exception(new RuntimeFailure("corrupted data in global cache"))
      }
    } else if (extractor.isCacheable &&
      extractor.getFingerprint.scope == ASTNode.CacheLevel.Event.scope) {
      eventContext.cacheGet(extractor.getFingerprint) {
        extractor.run(ctx)
      } match {
        case f: Future[AnyRef] => f
        case _ => Future.exception(new RuntimeFailure("corrupted data in event cache"))
      }
    } else if (extractor.isInstanceOf[Functor[T]]) {
      Future {
        val functor = extractor.asInstanceOf[Functor[T]]
        functor.apply(ctx)
      }
    } else {
      extractor.run(ctx)
    }
  }

  override def inputFeatureFunction: PartialFunction[String, AnyRef] = eventContext.inputFeatures

  override def addOutputFeature(namespace: AnyRef, fn: String, ft: Type, fv: AnyRef): Unit = {
    eventContext.addOutputFeature(namespace, rule.id, fn, ft, fv)
  }

  override def reportStat(namespace: AnyRef, statName: String, value: Long): Unit = {
    eventContext.reportStat(namespace, rule.id, statName, value)
  }

  override def reportFailure(namespace: AnyRef, category: String, t: Throwable): Unit = {
    eventContext.reportFailure(namespace, rule.id, category, t)
  }

  override val reportFunctionEvaluationConsumer: TriConsumer[String, ActionLevel, JList[_]] = {
    (functionName, actionLevel, args) =>
      eventContext.reportFunctionEvaluation(functionName, actionLevel, args.asScala, rule.id)
  }

  override def getStackFrames: JList[StackFrame] = Context.DEFAULT_STACK_FRAMES.get()

  override def applyActionRateLimit[R](
    context: Context[T],
    functionName: String,
    actionLevel: ActionLevel,
    func: () => Future[R],
    args: JList[_]
  ): Future[R] = {
    eventContext.applyActionRateLimit(
      context,
      functionName,
      actionLevel,
      rule.id,
      rule.actionLimits,
      func,
      args.asScala)
  }
}
