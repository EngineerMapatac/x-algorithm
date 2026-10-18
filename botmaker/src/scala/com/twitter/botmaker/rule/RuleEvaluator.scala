package com.twitter.botmaker.rule

import com.google.common.collect.ImmutableList
import com.twitter.botmaker.common.RatelimitedLogger
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.Context
import com.twitter.botmaker.Extractor
import com.twitter.botmaker.compiler.exceptions.FunctionFailure
import com.twitter.botmaker.compiler.exceptions.RuntimeFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.rule.RuleEvaluator._
import com.twitter.botmaker.rule.RuleResult.RuleResult
import com.twitter.botmaker.rule.thriftscala.BotMakerRule
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.finagle.stats.Counter
import com.twitter.finagle.stats.Stat
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.finagle.stats.Verbosity
import com.twitter.util.Duration
import com.twitter.util.Future
import com.twitter.util.Return
import com.twitter.util.Throw

case class RuleTimeoutException()
    extends RuntimeFailure("Future cancelled due to Rule timing out") {}

case class CompileError[T <: TwitterRuntime](t: Throwable)
    extends ASTNode[T]("CompilerError", ImmutableList.of[ASTNode[_]]) {

  override def getSignature: ASTNode.Signature = new ASTNode.Signature(
    ImmutableList.of[Type],
    Type.BOOLEAN
  )

  override def toExtractor: Extractor[T] = {
    new Extractor[T](this, ImmutableList.of[Extractor[_]]) {
      override def run(context: Context[T]): Future[AnyRef] = Future.value(boolean2Boolean(false))
    }
  }

  override protected def getCacheLevel = CacheLevel.Global
}

abstract class RuleEvaluator[T <: TwitterRuntime](
  rule: BotMakerRule,
  booleanAstRoot: ASTNode[T],
  actionAstRoot: ASTNode[T])
    extends RuleAst(rule, booleanAstRoot, actionAstRoot) {

  def evaluate(context: RuleContext[T], timeOutDuration: Duration): Future[Unit] = {

    run(context)
      .raiseWithin(context.getRuntime.timer, timeOutDuration, timeoutException)
      .handle {
        case _: RuleTimeoutException =>
          context.cancel()
      } transform { origTry =>
      context.addOutputFeature(
        RuleResultOutput,
        context.result.toString,
        Type.OBJECT,
        context.result
      )
      Future.const(origTry)
    }
  }

  def run(context: RuleContext[T]): Future[Unit]

  def hasCompilingFailure: Boolean
}

class FakeEvaluator[T <: TwitterRuntime](
  rule: BotMakerRule,
  val throwable: Throwable,
  val statsReceiver: StatsReceiver)
    extends RuleEvaluator[T](rule, CompileError[T](throwable), CompileError[T](throwable)) {

  private var counters = Map[(String, RuleResult), Counter]()
  private var stats = Map[String, Stat]()

  override def run(context: RuleContext[T]): Future[Unit] = {
    context.result = RuleResult.ABORTED
    counters = counter(
      statsReceiver,
      eventType,
      context.getTrackingId,
      context.result,
      counters
    )
    counters(eventType -> context.result).incr()
    context.reportStat(botEvaluationStats, context.result.toString, 1L)

    val latencyMs = context.nowMillis - context.startMillis
    stats = stat(statsReceiver, eventType, context.getTrackingId, stats)
    stats(eventType).add(latencyMs)
    context.reportStat(botEvaluationStats, botEvaluationLatencyStats, latencyMs)

    Future.Unit
  }

  override def hasCompilingFailure: Boolean = true
}

class CompiledEvaluator[T <: TwitterRuntime](
  rule: BotMakerRule,
  val booleanExtractor: Extractor[T],
  val actionExtractor: Extractor[T],
  val statsReceiver: StatsReceiver,
  val exceptionClassifier: Throwable => ExceptionCategory)
    extends RuleEvaluator[T](
      rule,
      booleanExtractor.getAstNode.asInstanceOf[ASTNode[T]],
      actionExtractor.getAstNode.asInstanceOf[ASTNode[T]]) {

  private var counters = Map[(String, RuleResult), Counter]()
  private var stats = Map[String, Stat]()

  override def run(context: RuleContext[T]): Future[Unit] = {
    Future.Unit flatMap { _ =>
      context.run(booleanExtractor)
    } transform {
      case Return(java.lang.Boolean.TRUE) =>
        context.result = RuleResult.VIOLATED
        Future.value(true)
      case Return(java.lang.Boolean.FALSE) =>
        context.result = RuleResult.UNVIOLATED
        Future.value(false)
      case Throw(t) =>
        val unwrappedThrowable = FunctionFailure.unwrap(t)
        val classified = exceptionClassifier(unwrappedThrowable)
        context.result =
          if (classified == Ignorable) RuleResult.ABORTED_IGNORED else RuleResult.ABORTED

        val runtime = context.getRuntime
        val exceptionType = t.getClass.getName
        logException(
          runtime.logger,
          classified,
          context.getTrackingId,
          s"go/bot/${runtime.name}/${context.getTrackingId} failed when evaluating the boolean expression.",
          t
        )
        context.reportStat(botEvaluationExceptions, exceptionType, 1L)
        context.reportFailure(botEvaluationExceptions, "condition", t)

        context.addOutputFeature(
          FailureOutput,
          "condition",
          FailureOutput.tupleType,
          FailureOutput.toTuple(t)
        )

        classified match {
          case Interruption | Ignorable => Future.exception(unwrappedThrowable)
          case NormalException => Future.False
        }

    } flatMap {
      case true =>
        context
          .run(actionExtractor).map(response =>
            context.addOutputFeature(
              ResponseOutput,
              ResponseOutput.ActionResponse,
              actionExtractor.getAstNode.getReturnType,
              response
            ))
      case false =>
        Future.Unit
    } rescue {
      case t: Throwable
          if context.result != RuleResult.ABORTED && context.result != RuleResult.ABORTED_IGNORED =>
        val unwrappedThrowable = FunctionFailure.unwrap(t)
        val classified = exceptionClassifier(unwrappedThrowable)
        context.result =
          if (classified == Ignorable) RuleResult.FAILED_IGNORED else RuleResult.FAILED

        val exceptionType = t.getClass.getName
        val runtime = context.getRuntime

        logException(
          runtime.logger,
          classified,
          context.getTrackingId,
          s"go/bot/${runtime.name}/${context.getTrackingId} failed when evaluating the action expression.",
          t
        )
        context.reportStat(botEvaluationExceptions, exceptionType, 1L)
        context.reportFailure(botEvaluationExceptions, "action", t)

        context.addOutputFeature(
          FailureOutput,
          "action",
          FailureOutput.tupleType,
          FailureOutput.toTuple(t)
        )

        classified match {
          case Interruption | Ignorable => Future.exception(unwrappedThrowable)
          case NormalException => Future.Unit
        }

    } transform { returnTry =>
      counters = counter(
        statsReceiver,
        eventType,
        context.getTrackingId,
        context.result,
        counters
      )
      counters(eventType -> context.result).incr()

      if (context.result == RuleResult.FAILED || context.result == RuleResult.FAILED_IGNORED) {
        counters = counter(
          statsReceiver,
          eventType,
          context.getTrackingId,
          RuleResult.VIOLATED,
          counters
        )
        counters(eventType -> RuleResult.VIOLATED).incr()
      }

      val latencyMs = context.nowMillis - context.startMillis
      stats = stat(statsReceiver, eventType, context.getTrackingId, stats)
      stats(eventType).add(latencyMs)

      context.reportStat(botEvaluationStats, context.result.toString, 1L)
      context.reportStat(botEvaluationStats, botEvaluationLatencyStats, latencyMs)
      Future.const(returnTry)
    }
  }

  override def hasCompilingFailure: Boolean = false

  private def logException(
    logger: RatelimitedLogger,
    exceptionCategory: ExceptionCategory,
    trackingId: Long,
    msg: String,
    t: Throwable
  ): Unit = {
    if (exceptionCategory == Ignorable) {
      logger.debug(Long.box(trackingId), msg, t)
    } else {
      logger.warning(Long.box(trackingId), msg, t)
    }
  }
}

object RuleEvaluator {

  val timeoutException: RuleTimeoutException = new RuleTimeoutException
  val botEvaluationStats = "bot_evaluation_stats"
  val botEvaluationExceptions = "bot_evaluation_exceptions"
  val botEvaluationLatencyStats = "latency"

  def counter(
    statsReceiver: StatsReceiver,
    eventType: String,
    ruleId: Long,
    ruleResult: RuleResult,
    map: Map[(String, RuleResult), Counter]
  ): Map[(String, RuleResult.RuleResult), Counter] = {
    if (map.contains((eventType, ruleResult))) {
      map
    } else {
      val verbosity =
        if (ruleResult == RuleResult.FAILED_IGNORED || ruleResult == RuleResult.ABORTED_IGNORED)
          Verbosity.Debug
        else Verbosity.Default
      map + ((eventType, ruleResult) -> statsReceiver.counter(
        verbosity,
        eventType,
        ruleId.toString,
        ruleResult.toString.toLowerCase
      ))
    }
  }

  def stat(
    statsReceiver: StatsReceiver,
    eventType: String,
    ruleId: Long,
    map: Map[String, Stat]
  ): Map[String, Stat] = {
    if (map.contains(eventType)) {
      map
    } else {
      map + (eventType -> statsReceiver.stat(
        eventType,
        ruleId.toString,
        "latency"
      ))
    }
  }
}
