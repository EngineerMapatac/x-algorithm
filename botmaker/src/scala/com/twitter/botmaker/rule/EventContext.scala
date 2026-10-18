package com.twitter.botmaker.rule

import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.exceptions.MissingFeatureException
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.rule.thriftscala.ActionLimit
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.util.Future

trait EventContext[T <: TwitterRuntime] {
  def runtime: T

  def eventType: String

  def startMillis: Long

  def skipRules: scala.collection.Set[Long]

  def inputFeatures: InputProvider

  final def isInputFeatureDefined(featureName: String): Boolean = {
    inputFeatures.isDefinedAt(featureName)
  }

  @throws[MissingFeatureException]
  final def getInputFeature(featureName: String): AnyRef = {
    inputFeatures.applyOrElse(featureName, (fn: String) => throw new MissingFeatureException(fn))
  }

  def cacheGet[CT](key: AnyRef)(f: => CT): CT
  def invalidateCache(key: AnyRef): Unit
  def cacheGetIfPresent(key: AnyRef): AnyRef
  def cachePut(key: AnyRef, value: AnyRef): Unit

  def addOutputFeature(namespace: AnyRef, ruleId: Long, fn: String, ft: Type, fv: AnyRef): Unit

  def reportStat(namespace: AnyRef, ruleId: Long, statName: String, value: Long): Unit

  def reportFailure(namespace: AnyRef, ruleId: Long, category: String, t: Throwable): Unit

  def reportFunctionEvaluation(
    functionName: String,
    actionLevel: ActionLevel,
    args: Seq[_],
    ruleId: Long
  ): Unit

  def applyActionRateLimit[R](
    context: Context[T],
    functionName: String,
    actionLevel: ActionLevel,
    ruleId: Long,
    actionLimits: collection.Map[String, Seq[ActionLimit]],
    func: () => Future[R],
    args: Seq[_]
  ): Future[R]
}

object EventContext {

  def apply[T <: TwitterRuntime](
    inputRuntime: T,
    eventName: String,
    cache: ExprCache = ExprCache(),
    skips: scala.collection.Set[Long] = Set.empty,
    inputFeaturesProvider: InputProvider = InputProvider.empty,
    outputCollector: OutputCollector = OutputCollector.empty,
    statCollector: StatCollector = StatCollector.empty,
    failureCollector: FailureCollector = FailureCollector.empty,
    profilingCollector: ProfilingCollector = ProfilingCollector.empty,
    actionRateLimiter: ActionRateLimiter[T] = ActionRateLimiter.empty[T]
  ): EventContext[T] = new EventContext[T] {

    override def runtime: T = inputRuntime

    override def eventType: String = eventName

    override def skipRules: scala.collection.Set[Long] = skips

    override val startMillis: Long = runtime.clock.nowMillis

    override def inputFeatures: InputProvider = inputFeaturesProvider

    override def cacheGet[CT](key: AnyRef)(f: => CT): CT = cache.get(key, f)
    override def invalidateCache(key: AnyRef): Unit = cache.invalidate(key)
    override def cacheGetIfPresent(key: AnyRef): AnyRef = cache.getIfPresent(key)
    override def cachePut(key: AnyRef, value: AnyRef): Unit = cache.put(key, value)

    override def addOutputFeature(
      namespace: AnyRef,
      ruleId: Long,
      fn: String,
      ft: Type,
      fv: AnyRef
    ): Unit = {
      outputCollector.addOutputFeature(namespace, eventType, ruleId, fn, ft, fv)
    }

    override def reportStat(
      namespace: AnyRef,
      ruleId: Long,
      statName: String,
      value: Long
    ): Unit = {
      statCollector.reportStat(namespace, eventType, ruleId, statName, value)
    }

    override def reportFailure(
      namespace: AnyRef,
      ruleId: Long,
      category: String,
      t: Throwable
    ): Unit = {
      failureCollector.reportFailure(namespace, eventType, ruleId, category, t)
    }

    override def reportFunctionEvaluation(
      functionName: String,
      actionLevel: ActionLevel,
      args: Seq[_],
      ruleId: Long
    ): Unit = {
      profilingCollector.reportFunctionEvaluation(functionName, actionLevel, args, ruleId)
    }

    override def applyActionRateLimit[R](
      context: Context[T],
      functionName: String,
      actionLevel: ActionLevel,
      ruleId: Long,
      actionLimits: collection.Map[String, Seq[ActionLimit]],
      func: () => Future[R],
      args: Seq[_]
    ): Future[R] = {
      actionRateLimiter.applyActionRateLimit(
        context,
        functionName,
        actionLevel,
        ruleId,
        actionLimits,
        func,
        args)
    }
  }
}
