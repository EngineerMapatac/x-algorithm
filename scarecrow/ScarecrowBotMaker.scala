package com.twitter.botmaker.app.scarecrow

import java.lang.{Boolean => JBoolean}
import java.lang.{Double => JDouble}
import java.lang.{Iterable => JIterable}
import java.lang.{Long => JLong}
import java.util.Collections
import java.util.{Collection => JCollection}
import java.util.{HashMap => JHashMap}
import java.util.{HashSet => JHashSet}
import java.util.{List => JList}
import java.util.{Map => JMap}
import java.util.{Set => JSet}
import java.util.function.{Function => JFunction}
import scala.collection.JavaConverters._
import scala.collection.compat._
import scala.collection.immutable.SortedSet
import scala.language.existentials
import com.google.common.collect.ImmutableMap
import com.google.common.collect.ImmutableSet
import com.google.common.collect.Lists
import com.twitter.botmaker._
import com.twitter.botmaker.app.module.AppBotMaker
import com.twitter.botmaker.app.module.AppEventOutputCollector
import com.twitter.botmaker.app.scarecrow.config.ScarecrowConfigs
import com.twitter.botmaker.app.scarecrow.function.AdminFetcher
import com.twitter.botmaker.app.scarecrow.function.Fetcher20
import com.twitter.botmaker.app.scarecrow.legacy._
import com.twitter.botmaker.app.scarecrow.migrate.MigrateConfig20
import com.twitter.botmaker.app.scarecrow.ratelimit.thriftscala.RateLimitedFunctionCall
import com.twitter.botmaker.app.thriftjava.SimpleListMatchResult
import com.twitter.botmaker.app.thriftjava.SimpleListTestRequest
import com.twitter.botmaker.app.thriftjava.SimpleListTestResponse
import com.twitter.botmaker.common.FeatureDataConverter
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.types.{Symbol => BMSymbol}
import com.twitter.botmaker.function.Constant
import com.twitter.botmaker.function.DerivedFeatureCall
import com.twitter.botmaker.function.misc.DebugOutput
import com.twitter.botmaker.loader.Schedule
import com.twitter.botmaker.rule.BotMaker.systemEvents
import com.twitter.botmaker.rule._
import com.twitter.botmaker.rule.function.ResponseCodes
import com.twitter.botmaker.rule.sys.PreEventOutputCollector
import com.twitter.botmaker.rule.thriftscala.ActionLimit
import com.twitter.botmaker.rule.thriftscala.BotMakerRule
import com.twitter.botmaker.rule.thriftscala.BotMakerRulesPackage
import com.twitter.botmaker.rule.thriftscala.RateLimitingStrategy
import com.twitter.botmaker.runtime._
import com.twitter.botmaker.runtime.thriftjava.BotMakerFunctionCall
import com.twitter.finagle.stats.Stat
import com.twitter.relevance.feature_store.thrift.FeatureData
import com.twitter.relevance.feature_store.thrift.FeatureValue
import com.twitter.safety_systems.simple_list.thriftscala.PatternType
import com.twitter.spam.botmaker_features.BotMakerFeatures
import com.twitter.util.Future
import com.twitter.util.Return
import com.twitter.util.Throw
import com.twitter.util.Time
import com.twitter.util.Try
import java.util.concurrent.ThreadLocalRandom
import javax.annotation.concurrent.NotThreadSafe
import com.twitter.logpipeline.client.common.EventPublisher
import com.twitter.logpipeline.client.EventPublisherManager
import com.twitter.logpipeline.client.serializers.EventLogMsgThriftStructSerializer

object RecordedActionOutput {
  override def toString: String = "RecordedActionOutput"

  def addInputFeature[E](builder: CompilerBuilder[E], systemEvent: String): CompilerBuilder[E] = {
    builder
      .addInputFeature(
        s"${systemEvent}.recordedActions",
        Type.listOf(Type.thriftOf(classOf[BotMakerRecordedAction]))
      )
  }
}

trait RecordedActionOutput extends EventOutputCollector {

  val recordedActions: JList[BotMakerRecordedAction] =
    Collections.synchronizedList(Lists.newArrayList[BotMakerRecordedAction])

  override def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ): Unit = {
    namespace match {
      case RecordedActionOutput =>
        fv match {
          case recordedAction: BotMakerRecordedAction =>
            recordedActions.add(recordedAction)
          case _ => ???
        }
      case _ => super.addOutputFeature(namespace, eventType, ruleId, fn, ft, fv)
    }
  }

  override def isDefinedAt(key: String): Boolean = key match {
    case "recordedActions" => true
    case _ => super.isDefinedAt(key)
  }

  override def apply(key: String): AnyRef = key match {
    case "recordedActions" => recordedActions
    case _ => super.apply(key)
  }
}

object BotUserActionOutput {
  override def toString: String = "BotUserActionOutput"

  def addInputFeature[E](builder: CompilerBuilder[E], systemEvent: String): CompilerBuilder[E] = {
    builder
      .addInputFeature(
        s"${systemEvent}.botUserActions",
        Type.listOf(Type.thriftOf(classOf[BotUserAction]))
      )
  }
}

trait BotUserActionOutput extends EventOutputCollector {

  val botUserActions: JList[BotUserAction] =
    Collections.synchronizedList(Lists.newArrayList[BotUserAction])

  override def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ): Unit = {
    namespace match {
      case BotUserActionOutput =>
        fv match {
          case action: BotUserAction =>
            botUserActions.add(action)
          case _ => ???
        }
      case _ => super.addOutputFeature(namespace, eventType, ruleId, fn, ft, fv)
    }
  }

  override def isDefinedAt(key: String): Boolean = key match {
    case "botUserActions" => true
    case _ => super.isDefinedAt(key)
  }

  override def apply(key: String): AnyRef = key match {
    case "botUserActions" => botUserActions
    case _ => super.apply(key)
  }
}

class ScarecrowOutputCollector
    extends AppEventOutputCollector
    with RuleResultOutput
    with DerivedFeatureData
    with FailureOutput
    with RecordedActionOutput
    with BotUserActionOutput {

  def recursivelyCollectResponseCode(
    collected: scala.collection.mutable.Set[ResponseCode],
    iterable: Iterable[_]
  ): Unit = {
    iterable.foreach {
      case code: ResponseCode => collected += code
      case jCollection: JCollection[_] =>
        recursivelyCollectResponseCode(collected, jCollection.asScala)
      case sCollection: Iterable[_] =>
        recursivelyCollectResponseCode(collected, sCollection)
      case _ =>
    }
  }

  override def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ): Unit = {
    namespace match {
      case ResponseOutput =>
        fv match {
          case responseCode: ResponseCode if fn == ResponseOutput.ActionResponse =>
            responseCodeCollection.put(ruleId, responseCode)
          case responseCodeSet: JSet[_] if fn == ResponseOutput.ActionResponse =>
            val collected = scala.collection.mutable.Set[ResponseCode]()
            recursivelyCollectResponseCode(collected, responseCodeSet.asScala)
            val responseCode = ResponseCodes.resolveResponseCode(collected)
            responseCodeCollection.put(ruleId, responseCode)
          case exemptionOverride: java.lang.Boolean if fn == ResponseOutput.ExemptionOverride =>
            if (exemptionOverride) {
              exemptionOverrideCollection.add(ruleId)
            }
          case _ if fn == ResponseOutput.ActionResponse =>
            responseCodeCollection.put(ruleId, ResponseCode.NONE)
          case _ => ()
        }

      case derivedFeature: com.twitter.botmaker.compiler.DerivedFeature =>
        val featureData = derivedFeature.getFixedReturnType match {
          case "StrValue" =>
            new FeatureData().setFeatureValue(FeatureValue.strValue(strValue(fv)))
          case "BooleanValue" =>
            new FeatureData().setFeatureValue(FeatureValue.booleanValue(booleanValue(fv)))
          case "DoubleValue" =>
            new FeatureData().setFeatureValue(FeatureValue.doubleValue(doubleValue(fv)))
          case "LongValue" =>
            new FeatureData().setFeatureValue(FeatureValue.longValue(longValue(fv)))
          case "#StrListValue" =>
            new FeatureData().setFeatureValue(FeatureValue.strListValue(strListValue(fv)))
          case "LongListValue" =>
            new FeatureData().setFeatureValue(FeatureValue.longListValue(longListValue(fv)))
          case "DoubleListValue" =>
            new FeatureData().setFeatureValue(FeatureValue.doubleListValue(doubleListValue(fv)))
          case "StrSetValue" =>
            new FeatureData().setFeatureValue(FeatureValue.strSetValue(strSetValue(fv)))
          case "LongSetValue" =>
            new FeatureData().setFeatureValue(FeatureValue.longSetValue(longSetValue(fv)))
          case "StrMapValue" =>
            new FeatureData().setFeatureValue(FeatureValue.strMapValue(strMapValue(fv)))
          case "DoubleMapValue" =>
            new FeatureData().setFeatureValue(FeatureValue.doubleMapValue(doubleMapValue(fv)))
          case _ =>
            FeatureDataConverter.toJavaWithDebugInfo(ft, fv)
        }
        derivedFeatureData.put(fn, featureData)

      case _ =>
        super.addOutputFeature(namespace, eventType, ruleId, fn, ft, fv)
    }
  }

  private def strValue(fv: AnyRef): String = {
    String.valueOf(fv)
  }

  private def booleanValue(fv: AnyRef): JBoolean = {
    fv.asInstanceOf[JBoolean]
  }

  private def longValue(fv: AnyRef): JLong = fv match {
    case l: JLong => l
    case x =>
      try {
        JLong.valueOf(x.toString)
      } catch {
        case _: NumberFormatException =>
          Long.box(JDouble.valueOf(x.toString).longValue())
      }
  }

  private def doubleValue(fv: AnyRef): JDouble = fv match {
    case d: JDouble => d
    case x => JDouble.valueOf(x.toString)
  }

  private def strListValue(fv: AnyRef): JList[String] = {
    fv.asInstanceOf[JIterable[AnyRef]].iterator.asScala.toSeq.map(strValue).asJava
  }

  private def longListValue(fv: AnyRef): JList[JLong] = {
    fv.asInstanceOf[JIterable[AnyRef]].iterator.asScala.toSeq.map(longValue).asJava
  }

  private def doubleListValue(fv: AnyRef): JList[JDouble] = {
    fv.asInstanceOf[JIterable[AnyRef]].iterator.asScala.toSeq.map(doubleValue).asJava
  }

  private def strSetValue(fv: AnyRef): JSet[String] = {
    fv.asInstanceOf[JIterable[AnyRef]].iterator.asScala.toSet.map(strValue).asJava
  }

  private def longSetValue(fv: AnyRef): JSet[JLong] = {
    fv.asInstanceOf[JIterable[AnyRef]].iterator.asScala.toSet.map(longValue).asJava
  }

  private def strMapValue(fv: AnyRef): JMap[String, String] = {
    fv.asInstanceOf[JMap[AnyRef, AnyRef]].asScala.map {
        case (k, v) => strValue(k) -> strValue(v)
      }.asJava
  }

  private def doubleMapValue(fv: AnyRef): JMap[String, JDouble] = {
    fv.asInstanceOf[JMap[AnyRef, AnyRef]].asScala.map {
        case (k, v) => strValue(k) -> doubleValue(v)
      }.asJava
  }

  private[scarecrow] var filteredRuleResponses: JSet[RuleResponse] = ImmutableSet.of[RuleResponse]

  def unfilteredRuleResponses: JSet[RuleResponse] = super.allRuleResponses

  override def allRuleResponses: JSet[RuleResponse] = filteredRuleResponses
}

object ScarecrowBotMaker {
  private final val MinTranspiledRuleId = 100000L
  private final val FutureTrue: Future[JBoolean] = Future.value(true)
  private final val FutureFalse: Future[JBoolean] = Future.value(false)
  private final val FutureOneLong: Future[JLong] = Future.value(1L)
  private final val FutureNull: Future[AnyRef] = Future.value(null)
  private def isTranspiledRule(rule: BotMakerRule) =
    rule.id >= ScarecrowBotMaker.MinTranspiledRuleId
  private final val EmptyRuleResponses = ImmutableSet.of(new RuleResponse(ResponseCode.NONE, 0))
}

class ScarecrowBotMaker(
  runtime: TwitterRuntime,
  fetcher10: com.twitter.botmaker.fetcher.Fetcher10,
  fetcher20: Fetcher20,
  adminFetcher: AdminFetcher,
  models: ModelRuntime,
  schedule: Schedule[BotMakerRulesPackage],
  appName: String,
  env: String,
  includeTranspiledRules: Boolean = false)
    extends AppBotMaker[ScarecrowConfigs, ScarecrowRuntime](
      runtime,
      schedule,
      if (includeTranspiledRules)
        identity
      else
        rulesPkg =>
          rulesPkg.copy(rules = rulesPkg.rules.filter(!ScarecrowBotMaker.isTranspiledRule(_)))
    ) {

  private val actionRateLimiterStats = runtime.statsReceiver.scope("action_rate_limiter")
  private val actionRateLimiterSuccessCounter = actionRateLimiterStats.counter("successes")
  private val actionRateLimiterFailureCounter = actionRateLimiterStats.counter("failures")
  private val botHaltedStateUpdateFailureCounter =
    actionRateLimiterStats.counter("state_update_failures")
  private val botHaltedStateUpdateSuccessCounter =
    actionRateLimiterStats.counter("state_update_successes")
  private val botActionOverLimitStats = actionRateLimiterStats.scope("bot_action_over_limit")

  private val logCategoryName = "botmaker_ratelimited_function_calls"
  private val publisher: EventPublisher[RateLimitedFunctionCall] =
    EventPublisherManager.buildScribeLogPipelinePublisher(
      logCategoryName,
      EventLogMsgThriftStructSerializer.getNewSerializer())

  override protected def createRuntime(registry: Seq[ServiceRegistry[Any]]): ScarecrowRuntime = {
    ScarecrowRuntime(
      this,
      fetcher10,
      fetcher20,
      adminFetcher,
      models,
      registry
    )
  }

  override protected def shouldEnableContextScope: BotMakerRule => Boolean = { rule: BotMakerRule =>
    rule.id < ScarecrowBotMaker.MinTranspiledRuleId && !"smyte_all".equals(
      rule.eventType) && !"smyte_no_delay".equals(rule.eventType)
  }

  override def packageNames: Seq[String] = {
    List(
      "com.twitter.botmaker.common",
      "com.twitter.botmaker.app.scarecrow.legacy",
      "com.twitter.botmaker.app.scarecrow.function",
      "com.twitter.botmaker.app.scarecrow.migrate.function"
    )
  }

  override protected def configCompilerBuilder: CompilerBuilder[ScarecrowConfigs] = {
    super.configCompilerBuilder
      .strictMode()
  }

  override protected def systemCompilerBuilder(
    systemConfigs: ScarecrowConfigs
  ): CompilerBuilder[ScarecrowRuntime] = {

    val builder = super.systemCompilerBuilder(systemConfigs)
    builder
      .addPackage("com.twitter.botmaker.app.scarecrow.sys")
      .addInputFeature(
        s"${systemEvents.preProcessEvent}.${InputProvider.INPUT_FEATURE_DATA}",
        Type.mapOf(Type.STRING, Type.thriftOf(classOf[FeatureData])))
      .addInputFeature(
        s"${systemEvents.postProcessEvent}.${InputProvider.INPUT_FEATURE_DATA}",
        Type.mapOf(Type.STRING, Type.thriftOf(classOf[FeatureData])))
      .strictMode()

    EventOutputFeatureData.addInputFeature(builder, BotMaker.systemEvents.postProcessEvent)
    RuleResultOutput.addInputFeature(builder, BotMaker.systemEvents.postProcessEvent)
    DerivedFeatureData.addInputFeature(builder, BotMaker.systemEvents.postProcessEvent)
    FailureOutput.addInputFeature(builder, BotMaker.systemEvents.postProcessEvent)
    ResponseOutput.addInputFeature(builder, BotMaker.systemEvents.postProcessEvent)
    RecordedActionOutput.addInputFeature(builder, BotMaker.systemEvents.postProcessEvent)
    BotUserActionOutput.addInputFeature(builder, BotMaker.systemEvents.postProcessEvent)
  }

  override def beforeStateUpdate(state: State): Unit = {
    val startMs = System.currentTimeMillis()
    RuleContainer.traverseExtractors(
      state.ruleContainer.evaluators,
      new SQRLVerdictTagVisitor(state.serviceRuntime.staticContext))
    val endMs = System.currentTimeMillis()
    logger.info(s"AST traversal took ${endMs - startMs} milliseconds")
  }

  override def checkAppEvent(
    eventType: String,
    inputFeatures: InputProvider
  ): Future[AppEventOutputCollector] = {

    val inputFeaturesWithEventThrift = getEventThriftInputProviderIfExists(inputFeatures)

    val sta = state

    val asyncEvent = eventType + "_async"
    if (sta.isEventDefined(asyncEvent)) {
      val asyncOutputCollector = new ScarecrowOutputCollector
      runEvent(
        sta,
        ExprCache(),
        asyncEvent,
        inputFeaturesWithEventThrift,
        asyncOutputCollector
      )
    }

    val outputCollector = new ScarecrowOutputCollector

    runEvent(
      sta,
      ExprCache(),
      eventType,
      inputFeaturesWithEventThrift,
      outputCollector
    )
  }

  override def applyActionRateLimit[R](
    context: Context[ScarecrowRuntime],
    functionName: String,
    actionLevel: ActionLevel,
    ruleId: Long,
    actionLimits: collection.Map[String, Seq[ActionLimit]],
    func: () => Future[R],
    args: Seq[_]
  ): Future[R] = {
    if (enableActionRateLimit(appName, env, functionName, actionLimits)) {
      val actionLimit = actionLimits(functionName).head
      val periodSecs = actionLimit.periodInMinutes * 60
      val limit = actionLimit.quota
      val strategy = actionLimit.strategy
      val version = actionLimit.version
      if (!waitingForHumanIntervention(ruleId, functionName, strategy, version)) {
        readAndUpdateActionLimitUsage(
          context,
          functionName,
          ruleId,
          func,
          periodSecs,
          limit,
          strategy,
          version,
          args)
      } else {
        logRateLimitedFunctionCalls[R](ruleId, Time.now.inMillis, functionName, args)
      }
    } else {
      func.apply()
    }
  }

  private def arlDeciderKeyForAppEnv(appName: String, env: String): String =
    s"enable_action_rate_limiter_${appName}_$env"

  private def enableActionRateLimit(
    appName: String,
    env: String,
    functionName: String,
    actionLimits: collection.Map[String, Seq[ActionLimit]]
  ): Boolean = {
    actionLimits.contains(functionName) &&
    fetcher10.getDecider.isAvailable(arlDeciderKeyForAppEnv(appName, env)) &&
    fetcher10.getDecider.isAvailableForRandomRecipient("enable_action_rate_limiter")
  }

  private def waitingForHumanIntervention(
    ruleId: Long,
    functionName: String,
    strategy: RateLimitingStrategy,
    version: Int
  ): Boolean = {
    strategy == RateLimitingStrategy.HaltUntilHumanIntervention &&
    botActionAlreadyHalted(ruleId, functionName, version)
  }

  private def logRateLimitedFunctionCalls[R](
    botId: Long,
    timestamp: Long,
    functionName: String,
    args: Seq[_],
  ): Future[R] = {
    val rateLimitedFunctionEvent = RateLimitedFunctionCall(
      botId,
      timestamp,
      functionName,
      args.map {
        case null => "null"
        case arg => arg.toString
      })
    publisher.publish(rateLimitedFunctionEvent).map(_ => null.asInstanceOf[R])
  }

  private def botActionAlreadyHalted(ruleId: Long, functionName: String, version: Int): Boolean = {
    val isHalted: Boolean = fetcher20.isBotActionHalted(ruleId, functionName, version)
    if (isHalted) {
      botActionOverLimitStats.counter(ruleId.toString, functionName).incr(1)
    }
    isHalted
  }

  private[this] def readAndUpdateActionLimitUsage[R](
    context: Context[ScarecrowRuntime],
    functionName: String,
    ruleId: Long,
    func: () => Future[R],
    periodSecs: Long,
    limit: Long,
    strategy: RateLimitingStrategy,
    version: Int,
    args: Seq[_]
  ): Future[R] = {
    fetcher10
      .getFeatureCountLite(
        context.getRuntime.getMigrateContext(context),
        Future.value(getActionRateLimiterFeatureName(functionName)),
        Future.value(ruleId.toString),
        Future.value(periodSecs),
        ScarecrowBotMaker.FutureFalse,
        ScarecrowBotMaker.FutureFalse
      ) flatMap (usage => {
      if (usage > limit) {
        Future.join(Seq(
          logRateLimitedFunctionCalls[R](ruleId, Time.now.inMillis, functionName, args),
          strategy match {
            case RateLimitingStrategy.TokenBucket =>
              Future.Unit
            case RateLimitingStrategy.HaltUntilHumanIntervention =>
              setBotActionHaltedState(appName, env, ruleId, functionName, version)
            case unknown =>
              logger.warning(s"Rate limiting strategy $unknown is not supported. " +
                "Falling back to token bucket.")
              Future.Unit
          }
        )) map { _ =>
          null.asInstanceOf[R]
        } onSuccess { _ =>
          botHaltedStateUpdateSuccessCounter.incr()
        } onFailure { t: Throwable =>
          botHaltedStateUpdateFailureCounter.incr()
          logger.warning("Error encountered while trying to set bot action to halted state. " +
            "Action rate limit was applied: " + t.getMessage)
        } ensure {
          botActionOverLimitStats.counter(ruleId.toString, functionName).incr(1)
        } rescue {
          case _ => ScarecrowBotMaker.FutureNull.asInstanceOf[Future[R]]
        }
      } else {
        fetcher10.incrementFeatureCountLite(
          Future.value(getActionRateLimiterFeatureName(functionName)),
          Future.value(ruleId.toString),
          ScarecrowBotMaker.FutureOneLong,
          ScarecrowBotMaker.FutureTrue
        )
        botActionOverLimitStats.counter(ruleId.toString, functionName).incr(0)
        func.apply()
      }
    }) onSuccess { _ =>
      actionRateLimiterSuccessCounter.incr()
    } onFailure { t: Throwable =>
      actionRateLimiterFailureCounter.incr()
      logger.warning(
        "Error encountered while applying action rate limit. " +
          "Action rate limit not applied: " + t.getMessage)
    } rescue {
      case _ => func.apply()
    }
  }

  private def getActionRateLimiterFeatureName(
    functionName: String
  ): String = "ActionRateLimiter_" + functionName

  private def setBotActionHaltedState(
    appName: String,
    env: String,
    ruleId: Long,
    functionName: String,
    version: Int
  ): Future[Unit] = {
    fetcher20.stratoFetcher20.setBotHaltedState(appName, env, ruleId, functionName, version)
  }

  private def getEventThriftInputProviderIfExists(inputFeatures: InputProvider): InputProvider = {
    if (!inputFeatures.isDefinedAt(BotMakerFeatures.eventThrift)) {
      inputFeatures
    } else {
      val eventThriftJson = inputFeatures(BotMakerFeatures.eventThrift).asInstanceOf[String]
      val featureData = new FeatureData().setFeatureValue(FeatureValue.strValue(eventThriftJson))
      val eventThriftObj = BotMakerHandler.BotMakerThriftEvent.deserialize(featureData)

      new InputProvider {
        override def isDefinedAt(x: String): Boolean = inputFeatures.isDefinedAt(x)

        override def apply(feature: String): AnyRef =
          if (!feature.equals(BotMakerFeatures.eventThrift)) {
            inputFeatures(feature)
          } else {
            eventThriftObj
          }
      }
    }
  }

  private def runEvent(
    sta: State,
    cache: ExprCache,
    eventType: String,
    inputFeatures: InputProvider,
    outputCollector: ScarecrowOutputCollector
  ): Future[ScarecrowOutputCollector] = {

    Stat.timeFuture(eventStats.stat(eventType, "latency")) {

      eventStats.counter(eventType, "requests").incr()
      val preOutput = new PreEventOutputCollector

      sta.checkSysEvent(
        BotMaker.systemEvents.preProcessEvent,
        cache,
        inputFeatures = {
          case "eventType" =>
            eventType
          case "rules" =>
            sta.rulesPackage.rules.asJava
          case InputProvider.INPUT_FEATURES
              if inputFeatures.isDefinedAt(InputProvider.INPUT_FEATURES) =>
            inputFeatures(InputProvider.INPUT_FEATURES)
        },
        outputCollector = preOutput
      ) onFailure { t: Throwable =>
        exceptionCounter(eventType, t, "exceptions", "pre").incr()
      } flatMap {
        case _ if preOutput.skipEvent =>
          eventStats.counter(eventType, "skipped").incr()
          Future.value(outputCollector)
        case _ =>
          sta.checkEvent(
            eventType = eventType,
            cache = cache,
            inputFeatures = inputFeatures,
            outputCollector = outputCollector,
            preOutput = preOutput,
            statCollector = this,
            failureCollector = this,
            profilingCollector = this,
            actionRateLimiter = this
          ) onFailure { t: Throwable =>
            exceptionCounter(eventType, t, "exceptions", "event").incr()
          } flatMap { _ =>
            outputCollector.unviolatedRules.asScala foreach { ruleId: Long =>
              outputCollector.responseCodeCollection.put(ruleId, ResponseCode.NONE)
            }

            val inputFeatureData: JMap[String, FeatureData] =
              if (inputFeatures.isDefinedAt(InputProvider.INPUT_FEATURE_DATA)) {
                inputFeatures(InputProvider.INPUT_FEATURE_DATA)
                  .asInstanceOf[JMap[String, FeatureData]]
              } else {
                ImmutableMap.of[String, FeatureData]
              }
            ExemptionUtil.filterExemptResponses(
              fetcher10,
              inputFeatureData,
              MigrateConfig20(sta.serviceRuntime),
              outputCollector.unfilteredRuleResponses
            ) map { filteredResponse =>
              outputCollector.filteredRuleResponses = if (filteredResponse.isEmpty) {
                ScarecrowBotMaker.EmptyRuleResponses
              } else {
                filteredResponse
              }
              filteredResponse.asScala.foreach(response =>
                eventStats.counter(eventType, "filtered", "responses", response.code.name()).incr())
              eventStats
                .counter(eventType, "responses", outputCollector.finalRuleResponse.code.name())
                .incr()
              outputCollector
            }
          } onFailure { t: Throwable =>
            exceptionCounter(eventType, t, "exceptions", "exempt").incr()
          } flatMap { _ =>
            val postOutputCollector = new EventOutputCollector {
              override def addOutputFeature(
                namespace: AnyRef,
                eventType: String,
                ruleId: Long,
                fn: String,
                ft: Type,
                fv: AnyRef
              ): Unit = namespace match {
                case DebugOutput.NAMESPACE =>
                  outputCollector.addOutputFeature(namespace, eventType, ruleId, fn, ft, fv)
                case _ => ()
              }
            }

            sta.checkSysEvent(
              BotMaker.systemEvents.postProcessEvent,
              cache,
              inputFeatures = {
                case "eventType" =>
                  eventType
                case "rules" =>
                  sta.rulesPackage.rules.asJava
                case InputProvider.INPUT_FEATURES
                    if inputFeatures.isDefinedAt(InputProvider.INPUT_FEATURES) =>
                  inputFeatures(InputProvider.INPUT_FEATURES)
                case InputProvider.INPUT_FEATURE_DATA
                    if inputFeatures.isDefinedAt(InputProvider.INPUT_FEATURE_DATA) =>
                  inputFeatures(InputProvider.INPUT_FEATURE_DATA)
                case key if outputCollector.isDefinedAt(key) =>
                  outputCollector(key)
              },
              postOutputCollector
            )
          } onFailure { t: Throwable =>
            exceptionCounter(eventType, t, "exceptions", "post").incr()
          } map { _ =>
            eventStats.counter(eventType, "successes").incr()
            outputCollector
          }
      }
    }
  }

  override def runExpression(
    eventType: String,
    expr: String
  )(
    inputFeatures: InputProvider
  ): Future[(Type, AnyRef)] = {
    val outputCollector = new ScarecrowOutputCollector
    super
      .runExpression(eventType, expr, outputCollector) {
        getEventThriftInputProviderIfExists(inputFeatures)
      }
  }

  private[this] implicit val patternTypeOrdering: Ordering[PatternType] =
    Ordering.by(patternType => patternType.value)

  override def testSimpleListValues(
    request: SimpleListTestRequest
  ): Try[SimpleListTestResponse] = {
    val inputValues = request.simpleListValues.asScala
      .map(listValue => listValue.value -> PatternType(listValue.patternType.getValue))
      .to(SortedSet)

    val matchesListResult =
      fetcher20.simpleListValuesMatch(
        inputValues,
        request.testInputs.asScala,
        extractFullMatchingString = true)
    val extractListResult =
      fetcher20.simpleListValuesMatch(
        inputValues,
        request.testInputs.asScala,
        extractFullMatchingString = false)

    (matchesListResult, extractListResult) match {
      case (Throw(ex), _) => Throw(ex)
      case (_, Throw(ex)) => Throw(ex)
      case (Return(mlr), Return(elr)) =>
        val perInputResponseList = mlr.zip(elr) map {
          case (matchesResponsePerInput, extractResponsePerInput) =>
            new SimpleListMatchResult()
              .setMatchesListExtracted(matchesResponsePerInput.asJava)
              .setExtractListExtracted(extractResponsePerInput.asJava)
        }
        Return(new SimpleListTestResponse().setPerInputMatches(perInputResponseList.asJava))
    }
  }

  override def reportFunctionEvaluation(
    functionName: String,
    actionLevel: ActionLevel,
    args: Seq[_],
    ruleId: Long
  ): Unit = {
    super.reportFunctionEvaluation(functionName, actionLevel, args, ruleId)
    val options = state.configContainer.systemConfigs.functionProfilingOptions.get(functionName)
    if (options != null && ThreadLocalRandom.current.nextDouble() < options.sampleRate) {
      val argsToSample = options.parametersToSample
      val toScribe = new BotMakerFunctionCall(functionName, ruleId, Time.now.inMillis)
      args.zipWithIndex.foreach {
        case (argValue, index) =>
          if (argsToSample.contains(index)) {
            val featureValue: FeatureValue = featureValueFromValue(argValue.asInstanceOf[AnyRef])
            val setThriftFunction: (BotMakerFunctionCall, FeatureValue) => BotMakerFunctionCall =
              setArgMap.getOrElse(
                index,
                (bmfc, _) => {
                  logger.warning(
                    functionName -> index,
                    s"No function built in for arg '$index' on function $functionName - not setting value"
                  )
                  bmfc
                }
              )
            setThriftFunction.apply(toScribe, featureValue)
          }
      }
      fetcher20.reportBotmakerFunctionCall(toScribe)
    }
  }

  private[this] val setArgMap: Map[
    Int,
    (BotMakerFunctionCall, FeatureValue) => BotMakerFunctionCall
  ] = Map(
    0 -> ((bmfc: BotMakerFunctionCall, theVal) => bmfc.setArg0(theVal)),
    1 -> ((bmfc: BotMakerFunctionCall, theVal) => bmfc.setArg1(theVal)),
    2 -> ((bmfc: BotMakerFunctionCall, theVal) => bmfc.setArg2(theVal)),
    3 -> ((bmfc: BotMakerFunctionCall, theVal) => bmfc.setArg3(theVal)),
    4 -> ((bmfc: BotMakerFunctionCall, theVal) => bmfc.setArg4(theVal)),
    5 -> ((bmfc: BotMakerFunctionCall, theVal) => bmfc.setArg5(theVal)),
    6 -> ((bmfc: BotMakerFunctionCall, theVal) => bmfc.setArg6(theVal)),
    7 -> ((bmfc: BotMakerFunctionCall, theVal) => bmfc.setArg7(theVal)),
    8 -> ((bmfc: BotMakerFunctionCall, theVal) => bmfc.setArg8(theVal)),
    9 -> ((bmfc: BotMakerFunctionCall, theVal) => bmfc.setArg9(theVal)),
    10 -> ((bmfc: BotMakerFunctionCall, theVal) => bmfc.setArg10(theVal)),
    11 -> ((bmfc: BotMakerFunctionCall, theVal) => bmfc.setArg11(theVal))
  )

  private[this] def featureValueFromValue(argValue: AnyRef): FeatureValue = {
    argValue match {
      case value: java.lang.Short => FeatureValue.shortValue(value)
      case value: java.lang.Integer => FeatureValue.intValue(value)
      case value: java.lang.Long => FeatureValue.longValue(value)
      case value: java.lang.Double => FeatureValue.doubleValue(value)
      case value: java.lang.Boolean => FeatureValue.booleanValue(value)
      case value: java.lang.String => FeatureValue.strValue(value)
      case map: java.util.Map[_, _] =>
        FeatureValue.strMapValue(map.asScala.map {
          case (key, v) => String.valueOf(key) -> String.valueOf(v)
        }.asJava)
      case list: java.util.List[_] =>
        FeatureValue.strListValue(list.asScala.map(String.valueOf).asJava)
      case set: java.util.Set[_] => FeatureValue.strSetValue(set.asScala.map(String.valueOf).asJava)
      case value if value == null => null
      case _ =>
        logger.warning(
          "featureValueFromValue" -> argValue.getClass,
          s"Unsupported type to convert to FeatureValue for function sampling - ${argValue.getClass}"
        )
        null
    }
  }

  object SQRLVerdictTagVisitor {
    val DerivedFeatureCallClassName: String = classOf[DerivedFeatureCall].getSimpleName
    val JHashSetFactory: JFunction[String, JHashSet[Long]] = (_: String) => new JHashSet[Long]()
  }

  @NotThreadSafe
  class SQRLVerdictTagVisitor(staticContext: StaticContext) extends ExtractorVisitorUnit {
    private val map: JMap[String, JSet[Long]] = new JHashMap[String, JSet[Long]]()
    private var currentRuleId: Long = 0L
    private val _visitor: PartialFunction[Extractor[_], Unit] =
      new PartialFunction[Extractor[_], Unit] {
        override def apply(extractor: Extractor[_]): Unit = {
          if (extractor.getAstNode.getClassName == SQRLVerdictTag.funcName) {
            getVerdictFromExtractor(extractor).foreach { symbol =>
              updateVerdictMap(symbol.getValue)
            }
          }
        }
        override def isDefinedAt(extractor: Extractor[_]): Boolean =
          extractor.getAstNode.getClassName != SQRLVerdictTagVisitor.DerivedFeatureCallClassName
      }
    private def getVerdictFromExtractor(extractor: Extractor[_]): Option[BMSymbol] = {
      Try {
        extractor.getAstNode.getChildren
          .get(0)
          .asInstanceOf[Constant]
          .getValue
          .asInstanceOf[BMSymbol]
      }.onFailure { t: Throwable =>
        logger.warning(t, s"SQRLVerdictTagVisitor failed when visiting ${currentRuleId}", t)
      }.toOption
    }

    private def updateVerdictMap(verdict: String): Unit = {
      map.computeIfAbsent(verdict, SQRLVerdictTagVisitor.JHashSetFactory)
      val set = map.get(verdict)
      set.add(currentRuleId)
      logger.info(s"rule ${currentRuleId} contains verdict $verdict")
    }

    override def onRuleInit(rule: BotMakerRule): Unit = {
      currentRuleId = rule.id
      logger.debug(s"[SQRLVerdictTagVisitor] visiting rule ${currentRuleId}")
    }

    override def onRuleExit(): Unit = {
      currentRuleId = 0
    }

    override def visitor: PartialFunction[Extractor[_], Unit] = _visitor

    override def onCompletion(): Unit = {
      staticContext.verdictSideEffects = map
    }
  }
}
