package com.twitter.botmaker.rule

import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.Extractor
import com.twitter.botmaker.antlr.BotMakerLexer
import com.twitter.botmaker.common.RatelimitedLogger
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerDoc
import com.twitter.botmaker.compiler.Compiler
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.compiler.CompilerContext
import com.twitter.botmaker.compiler.DerivedFeatureProvider
import com.twitter.botmaker.compiler.Fingerprint
import com.twitter.botmaker.compiler.Parser
import com.twitter.botmaker.function.misc.DebugOutput
import com.twitter.botmaker.loader.Loader
import com.twitter.botmaker.loader.Schedule
import com.twitter.botmaker.loader.Scheduler
import com.twitter.botmaker.rule.BotMaker.systemEvents
import com.twitter.botmaker.rule.config.SystemConfigs
import com.twitter.botmaker.rule.sys.PreEventOutput
import com.twitter.botmaker.rule.sys.PreEventOutputCollector
import com.twitter.botmaker.rule.thriftscala.ActionLimit
import com.twitter.botmaker.rule.thriftscala.BotMakerDerivedFeature
import com.twitter.botmaker.rule.thriftscala.BotMakerRule
import com.twitter.botmaker.rule.thriftscala.BotMakerRulesPackage
import com.twitter.botmaker.runtime.thriftjava.BotMakerEventTypeConfig
import com.twitter.botmaker.runtime.thriftjava.TeamConfig
import com.twitter.botmaker.runtime.RegexContainer
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.ServiceRegistry
import com.twitter.botmaker.runtime.ServiceRuntime
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.finagle.stats.Counter
import com.twitter.finagle.stats.Stat
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.finagle.stats.Verbosity
import com.twitter.relevance.feature_store.thrift.FeatureData
import com.twitter.util._
import org.antlr.runtime.tree.Tree
import scala.collection.JavaConverters._
import scala.collection.mutable.ArrayBuffer
import scala.reflect.ClassTag

class BotMakerRuntime[CT <: SystemConfigs](
  val botmaker: BotMaker[CT, _],
  val registry: Seq[ServiceRegistry[Any]])
    extends ServiceRuntime[CT]
    with RegexContainer
    with EventProcessor {

  type REGISTRY_MAP = Map[String, ServiceRegistry[Any]]

  override type _CONFIGS_TYPE = CT

  override def runtime: TwitterRuntime = botmaker.runtime

  private val registryMap: Map[Class[_ <: ServiceRegistry[Any]], REGISTRY_MAP] =
    registry groupBy { r =>
      r.getClass
    } map {
      case (c, seq) =>
        c -> seq.map(r => r.name -> r).toMap
    } toMap

  def serviceRegistry[T <: ServiceRegistry[Any]](
    name: String
  )(
    implicit ctag: ClassTag[T]
  ): Option[T] = {
    val clazz = ctag.runtimeClass.asInstanceOf[Class[T]]
    registryMap
      .get(clazz)
      .flatMap(_.get(name))
      .asInstanceOf[Option[T]]
  }

  override def cacheGet(key: AnyRef)(f: => AnyRef): AnyRef = botmaker.globalCache.get(key, f)

  override def processEvent(
    eventType: String,
    inputFeatures: PartialFunction[String, AnyRef],
    outputCollector: OutputCollector
  ): Future[Unit] = {
    botmaker.processEvent(eventType, inputFeatures, outputCollector)
  }

  override def checkEvent[T <: EventOutputCollector](
    eventType: String,
    inputFeatures: InputProvider,
    outputCollector: T
  ): Future[T] = {
    botmaker.checkEvent(eventType, inputFeatures, outputCollector)
  }

  override def debug(dbg: RuntimeDebugger): Unit = {

    super.debug(dbg)

    dbg.open("config", botmaker.state.configContainer) foreach {
      botmaker.state.configContainer.systemConfigs.debug(_)
    }

    def fn(n: String): String =
      if (n.endsWith("Registry")) {
        n.substring(0, n.length - 8)
      } else {
        n
      }

    dbg.open("service", registry) foreach { d =>
      registryMap foreach {
        case (c, map) =>
          val name = fn(c.getSimpleName)
          d.open(name, map) foreach { di =>
            map foreach {
              case (n, r) =>
                di.open(n, r) foreach { r.debug }
            }
          }

      }
    }
  }
}

object BotMaker {
  val emptyRulesPackage: BotMakerRulesPackage = BotMakerRulesPackage()

  object systemEvents {
    val configEvent = "system_config"
    val preProcessEvent = "system_preprocess_event"
    val postProcessEvent = "system_postprocess_event"
    val reportStat = "system_report_stat"
    val reportFailure = "system_report_failure"
    val validateEvent = "system_validate_event"
  }

  val systemEventPrefix = "system_"

  def isFunctionOrFeatureUsed(
    rulesPackage: BotMakerRulesPackage,
    functionOrFeatureName: String
  ): Boolean = {
    rulesPackage.rules.exists(ruleUsesFuncOrDf(_, functionOrFeatureName)) ||
    rulesPackage.derivedFeatures.exists(dfUsesFuncOrDf(_, functionOrFeatureName))
  }

  def getFunctionOrFeatureDirectUsages(
    rulesPackage: BotMakerRulesPackage,
    functionOrFeatureName: String
  ): UsageInfo = {
    val ruleIds: Set[Long] =
      rulesPackage.rules.filter(ruleUsesFuncOrDf(_, functionOrFeatureName)).map(_.id).toSet
    val dfNames: Set[String] =
      rulesPackage.derivedFeatures
        .filter(dfUsesFuncOrDf(_, functionOrFeatureName)).map(_.name).toSet
    UsageInfo(ruleIds, dfNames)
  }

  private def ruleUsesFuncOrDf(rule: BotMakerRule, funcOrDfName: String): Boolean = {
    treeContainsFuncOrFeature(Parser.createParseTree(rule.booleanExpression), funcOrDfName) ||
    treeContainsFuncOrFeature(Parser.createParseTree(rule.actionExpression), funcOrDfName)
  }

  private def dfUsesFuncOrDf(df: BotMakerDerivedFeature, funcOrDfName: String): Boolean = {
    treeContainsFuncOrFeature(
      Parser.createParseParamsTree(df.featureExpression).getFirst,
      funcOrDfName)
  }

  private def treeContainsFuncOrFeature(tree: Tree, funcOrFeatureName: String): Boolean = {
    treeIsFuncOrFeature(tree, funcOrFeatureName) ||
    (0 until tree.getChildCount).exists { i =>
      treeContainsFuncOrFeature(tree.getChild(i), funcOrFeatureName)
    }
  }

  private def treeIsFuncOrFeature(tree: Tree, funcOrFeatureName: String): Boolean = {
    (tree.getType == BotMakerLexer.FEATURE && tree.getText == funcOrFeatureName) ||
    (tree.getType == BotMakerLexer.FUNCTION && getFuncName(tree) == funcOrFeatureName)
  }

  private def getFuncName(funcTree: Tree): String = {
    val funcNameTree: Tree = funcTree.getChild(0)
    val featureTree: Tree = funcNameTree.getChild(0)
    featureTree.getText
  }
}

abstract class BotMaker[CT <: SystemConfigs, RT <: BotMakerRuntime[CT]](
  val runtime: TwitterRuntime,
  val schedule: Schedule[BotMakerRulesPackage],
  val filterRules: BotMakerRulesPackage => BotMakerRulesPackage = identity
)(
  implicit ct: Manifest[CT],
  rt: Manifest[RT])
    extends StatCollector
    with FailureCollector
    with ProfilingCollector
    with ActionRateLimiter[RT]
    with Closable {

  private[this] val statsReceiver = runtime.statsReceiver

  private[this] val packageStats = statsReceiver.scope("package")
  private[this] val loadSkip = packageStats.counter("skipped")
  private[this] val loadSuccess = packageStats.counter("successes")
  private[this] val loadFailure = packageStats.counter("failures")
  private[this] val initStateFailure = packageStats.counter("init_state_failures")
  private[this] val stateTransitionFailure = packageStats.counter("transition_failures")
  @volatile
  private[this] var compilationErrors: Int = 0
  private[this] val compilationErrorGauge = packageStats.addGauge("compilation_errors") {
    compilationErrors
  }

  protected val eventStats: StatsReceiver = statsReceiver.scope("event")
  protected val userImpactByRuleStats: StatsReceiver = statsReceiver
    .scope("user_impact", "by_rule")

  protected[rule] lazy val globalCache: ExprCache = ExprCache()
  protected lazy val logger: RatelimitedLogger = runtime.logger
  val scheduler: Scheduler = new Scheduler(runtime.timer)

  protected def createRuntime(registry: Seq[ServiceRegistry[Any]]): RT

  protected def shouldEnableContextScope: BotMakerRule => Boolean = _ => true

  protected def configCompilerBuilder: CompilerBuilder[CT] = {
    val builder = Compiler
      .builder(ct.runtimeClass.asInstanceOf[Class[CT]])
      .addPackage("com.twitter.botmaker.config")
      .addPackage("com.twitter.botmaker.runtime.config")
      .addPackage("com.twitter.botmaker.rule.config")

    builder.addInputFeature(s"${systemEvents.configEvent}.APP_NAME", Type.STRING)
    runtime.info foreach { _ =>
      builder
        .addInputFeature(s"${systemEvents.configEvent}.ENV_NAME", Type.STRING)
        .addInputFeature(s"${systemEvents.configEvent}.CLUSTER_NAME", Type.STRING)
        .addInputFeature(s"${systemEvents.configEvent}.SERVICE_NAME", Type.STRING)
        .addInputFeature(s"${systemEvents.configEvent}.ROLE_NAME", Type.STRING)
        .addInputFeature(s"${systemEvents.configEvent}.SHARD_ID", Type.LONG)
        .addInputFeature(s"${systemEvents.configEvent}.IS_ADMIN_INSTANCE", Type.BOOLEAN)
    }

    builder
  }

  protected def systemCompilerBuilder(systemConfigs: CT): CompilerBuilder[RT] = {
    val ruleType = Type.thriftStructOf(classOf[BotMakerRule])
    systemConfigs
      .compilerBuilder[RT]
      .addPackage("com.twitter.botmaker.rule.function")
      .addPackage("com.twitter.botmaker.rule.sys")
      .addInputFeature(s"${systemEvents.preProcessEvent}.eventType", Type.STRING)
      .addInputFeature(s"${systemEvents.preProcessEvent}.rules", Type.listOf(ruleType))
      .addInputFeature(
        s"${systemEvents.preProcessEvent}.${InputProvider.INPUT_FEATURES}",
        Type.mapOf(Type.STRING, Type.OBJECT)
      )
      .addInputFeature(s"${systemEvents.postProcessEvent}.eventType", Type.STRING)
      .addInputFeature(s"${systemEvents.postProcessEvent}.rules", Type.listOf(ruleType))
      .addInputFeature(
        s"${systemEvents.postProcessEvent}.${InputProvider.INPUT_FEATURES}",
        Type.mapOf(Type.STRING, Type.OBJECT)
      )
  }

  protected def compilerBuilder(systemConfigs: CT): CompilerBuilder[RT] = {
    systemConfigs
      .compilerBuilder[RT]
      .addPackage("com.twitter.botmaker.rule.function")
      .addInputFeatures(systemConfigs.inputFeatureTypes)
  }

  protected def beforeStateUpdate(state: BotMaker.this.State): Unit = Unit

  protected def classifyException: Throwable => ExceptionCategory = _ => NormalException

  trait CompiledState {

    def rulesPackage: BotMakerRulesPackage

    def fingerprint: Fingerprint

    def configContainer: ConfigContainer[CT]

    def systemContainer: RuleContainer[RT]

    def ruleContainer: RuleContainer[RT]

    def derivedFeatures: DerivedFeatureContainer

    def isEventDefined(eventType: String): Boolean = {
      if (eventType.startsWith(BotMaker.systemEventPrefix)) {
        systemContainer.isEventDefined(eventType)
      } else {
        ruleContainer.isEventDefined(eventType)
      }
    }
  }

  case class State(
    rulesPackage: BotMakerRulesPackage,
    fingerprint: Fingerprint,
    configContainer: ConfigContainer[CT],
    systemContainer: RuleContainer[RT],
    ruleContainer: RuleContainer[RT],
    derivedFeatures: DerivedFeatureContainer,
    serviceRuntime: RT)
      extends CompiledState {

    private[this] val ruleContainerExtractor =
      new DefaultRuleAstExtractorForAnalysis(ruleContainer)
    private[this] val systemContainerExtractor =
      new DefaultRuleAstExtractorForAnalysis(systemContainer)

    def hasCompilingFailure: Boolean = {
      ruleContainer.hasCompilingFailure || systemContainer.hasCompilingFailure
    }

    def isEmpty: Boolean = {
      rulesPackage == BotMaker.emptyRulesPackage
    }

    def dump(): Unit = {
      val numSystemRules = systemContainer.numRules
      val numRules = ruleContainer.numRules
      val numMappedRules = ruleContainer.numMappedRules
      val failedRules = ruleContainer.failedRules
      val numEvents = ruleContainer.numEvents

      logger.info(
        s"loaded $numRules rules, ${failedRules.size} failed, $numEvents events, $numMappedRules rules after event mapping, $numSystemRules system rules"
      )

      compilationErrors = failedRules.size

      failedRules foreach { r =>
        logger.warning(s"go/bot/${runtime.name}/${r.id}: ${r.throwable.getMessage}")
      }
    }

    def getBotMakerDoc(category: String): Seq[BotMakerDoc] = category match {
      case "config" =>
        configContainer.compiler.getBotMakerDocs.asScala
      case "system" =>
        systemContainer.compiler.getBotMakerDocs.asScala
      case "regular" =>
        ruleContainer.compiler.getBotMakerDocs.asScala
      case _ =>
        ruleContainer.compiler.getBotMakerDocs.asScala
    }

    def getEventTypes(): Map[String, BotMakerEventTypeConfig] = {
      configContainer.systemConfigs.getEventTypes
    }

    def getTeams(): Seq[TeamConfig] = {
      configContainer.systemConfigs.getTeams
    }

    def analyzeRuleCode(): Map[Long, Map[String, FeatureData]] = {
      analyzeCodeForContainer(ruleContainer, ruleContainerExtractor) ++
        analyzeCodeForContainer(systemContainer, systemContainerExtractor)
    }

    private def analyzeCodeForContainer(
      ruleContainer: RuleContainer[_],
      extractor: RuleAstExtractorForAnalysis
    ): Map[Long, Map[String, FeatureData]] = {
      ruleContainer.rules
        .map(ruleId => ruleId -> extractor.analyzeRule(ruleId).getOrElse(Map.empty))
        .toMap
    }

    def validatePackage: Future[Unit] = {

      val output = new EventOutputCollector with RuleResultOutput
      checkSysEvent(
        BotMaker.systemEvents.validateEvent,
        ExprCache.empty,
        inputFeatures = {
          case "rules" =>
            rulesPackage.rules.asJava
        },
        output
      ) map { _ =>
        val failedRules = output.abortedRules.asScala ++ output.actionFailedRules.asScala
        if (failedRules.nonEmpty) {
          val msg = failedRules map { rid =>
            val rname = rulesPackage.rules.filter(_.id == rid).map(_.name).head
            s"Rule $rid: $rname"
          }
          throw new SemanticCheckFailure(s"Failed to validate. $msg")
        }
      }
    }

    def compile(rulesPkg: BotMakerRulesPackage): Option[BotMaker.this.CompiledState] = {
      val filteredRulesPkg = filterRules(rulesPkg)
      Fingerprint.of(filteredRulesPkg) match {
        case fpt if fpt == fingerprint =>
          logger.info(s"compile package: sig=$fingerprint, newSig=$fpt, skipped")
          None

        case fpt =>
          logger.info(s"compile package: sig=$fingerprint, newSig=$fpt, compiling")

          val systemBotMakerRules = ArrayBuffer[BotMakerRule]()
          val configBotMakerRules = ArrayBuffer[BotMakerRule]()
          val botMakerRules = ArrayBuffer[BotMakerRule]()

          filteredRulesPkg.rules foreach {
            case r if r.eventType == BotMaker.systemEvents.configEvent =>
              configBotMakerRules += r
            case r if r.eventType.startsWith(BotMaker.systemEventPrefix) =>
              systemBotMakerRules += r
            case r =>
              botMakerRules += r
          }

          logger.info("putting DFs into container...")
          val derivedFeatureContainer = DerivedFeatureContainer(
            derivedFeatures,
            filteredRulesPkg.derivedFeatures
          )
          logger.info("finished putting all DFs into container")

          logger.info("compiling config bots...")
          val sysConfigs = ConfigContainer(
            configContainer,
            configBotMakerRules,
            derivedFeatureContainer,
            statsReceiver.scope("config")
          )
          logger.info("finished compiling all config bots")

          logger.info("compiling system bots...")
          val sysRules = RuleContainer(
            container = systemContainer,
            compiler = systemCompilerBuilder(sysConfigs.systemConfigs).build(),
            botMakerRules = systemBotMakerRules,
            derivedFeatures = derivedFeatureContainer,
            exceptionClassifier = classifyException,
            statsReceiver = statsReceiver.scope("rule")
          )
          logger.info("finished compiling all system bots")

          if (sysRules.failedRules.nonEmpty) {
            val msg = sysRules.failedRules.map { r =>
              s"${r.id}: ${r.throwable}"
            } mkString "\n"
            throw new SemanticCheckFailure(s"failed to compile system rules:\n$msg")
          }

          logger.info("compiling regular bots...")
          val regularRules = RuleContainer(
            container = ruleContainer,
            compiler = compilerBuilder(sysConfigs.systemConfigs).build(),
            botMakerRules = botMakerRules,
            derivedFeatures = derivedFeatureContainer,
            exceptionClassifier = classifyException,
            enableContextScope = shouldEnableContextScope,
            eventMappings = sysConfigs.systemConfigs.eventMappings,
            statsReceiver = statsReceiver.scope("rule")
          )
          logger.info("finished compiling all regular bots")

          val compiledState = new CompiledState {
            override def rulesPackage: BotMakerRulesPackage = filteredRulesPkg

            override def fingerprint: Fingerprint = fpt

            override def configContainer: ConfigContainer[CT] = sysConfigs

            override def systemContainer: RuleContainer[RT] = sysRules

            override def ruleContainer: RuleContainer[RT] = regularRules

            override def derivedFeatures: DerivedFeatureContainer = derivedFeatureContainer
          }

          Some(compiledState)
      }
    }

    def checkEvent(
      eventType: String,
      cache: ExprCache,
      inputFeatures: InputProvider,
      outputCollector: OutputCollector,
      preOutput: PreEventOutput,
      statCollector: StatCollector,
      failureCollector: FailureCollector,
      profilingCollector: ProfilingCollector,
      actionRateLimiter: ActionRateLimiter[RT]
    ): Future[Unit] = {
      if (ruleContainer.isEventDefined(eventType)) {
        val eventContext = EventContext(
          inputRuntime = serviceRuntime,
          eventName = eventType,
          cache = cache,
          skips = preOutput.skipRules,
          inputFeaturesProvider = inputFeatures,
          outputCollector = outputCollector,
          statCollector = statCollector,
          failureCollector = failureCollector,
          profilingCollector = profilingCollector,
          actionRateLimiter = actionRateLimiter
        )

        val eventEvaluator = ruleContainer.getEventEvaluator(eventType)

        val timeout = if (preOutput.eventTimeout != Duration.Top) {
          preOutput.eventTimeout
        } else {
          configContainer.systemConfigs.eventTimeouts.getOrDefault(eventType, Duration.Top)
        }

        eventEvaluator.evaluate(eventContext, timeout)
      } else {
        Future.value(outputCollector)
      }
    }

    def checkSysEvent(
      eventType: String,
      exprCache: ExprCache,
      inputFeatures: InputProvider,
      outputCollector: OutputCollector = OutputCollector.empty,
      statCollector: StatCollector = StatCollector.empty,
      failureCollector: FailureCollector = FailureCollector.empty,
      profilingCollector: ProfilingCollector = ProfilingCollector.empty,
      actionRateLimiter: ActionRateLimiter[RT] = ActionRateLimiter.empty[RT]
    ): Future[Unit] = {
      if (systemContainer.isEventDefined(eventType)) {
        val eventContext = EventContext(
          inputRuntime = serviceRuntime,
          eventName = eventType,
          cache = exprCache,
          inputFeaturesProvider = inputFeatures,
          outputCollector = outputCollector,
          statCollector = statCollector,
          failureCollector = failureCollector,
          profilingCollector = profilingCollector,
          actionRateLimiter = actionRateLimiter
        )

        val eventEvaluator = systemContainer.getEventEvaluator(eventType)
        val timeout =
          configContainer.systemConfigs.eventTimeouts.getOrDefault(eventType, Duration.Top)
        eventEvaluator.evaluate(eventContext, timeout)
      } else {
        Future.Done
      }
    }
  }

  @volatile
  private[this] var _state = {
    val rulesPackage = BotMakerRulesPackage()

    val systemConfigs = ct.runtimeClass.newInstance().asInstanceOf[CT]

    val configCompiler = configCompilerBuilder.build
    val systemCompiler = systemCompilerBuilder(systemConfigs).build
    val compiler = compilerBuilder(systemConfigs).build
    val runtimeInputFeatures = Map[String, AnyRef](
      "APP_NAME" -> runtime.name,
      "ENV_NAME" -> runtime.info.map(_.environment).getOrElse(""),
      "CLUSTER_NAME" -> runtime.info.map(_.cluster).getOrElse(""),
      "SERVICE_NAME" -> runtime.info.map(_.serviceName).getOrElse(""),
      "ROLE_NAME" -> runtime.info.map(_.roleName).getOrElse(""),
      "SHARD_ID" -> long2Long(runtime.info.map(_.shardId).getOrElse(0).toLong),
      "IS_ADMIN_INSTANCE" -> boolean2Boolean(runtime.info.map(_.isAdminInstance).getOrElse(false))
    )

    val serviceRuntime = createRuntime(Nil)

    val sta = State(
      rulesPackage = rulesPackage,
      fingerprint = Fingerprint.of(rulesPackage),
      configContainer = ConfigContainer(configCompiler, runtimeInputFeatures),
      systemContainer = RuleContainer(systemCompiler),
      ruleContainer = RuleContainer(compiler),
      derivedFeatures = DerivedFeatureContainer.empty,
      serviceRuntime = serviceRuntime
    )
    this.beforeStateUpdate(sta)
    sta
  }

  def state: BotMaker.this.State = _state

  private[this] def state_=(state: BotMaker.this.State): Unit = {
    beforeStateUpdate(state)
    _state = state
  }

  protected def load(rulesPackage: BotMakerRulesPackage): Unit = synchronized {
    try {
      val sta = state
      sta.compile(rulesPackage) match {
        case Some(compiledState) =>
          val registry = sta.serviceRuntime.build(
            compiledState.configContainer.systemConfigs,
            statsReceiver.scope("service")
          )

          val serviceRuntime = createRuntime(registry)

          state = State(
            rulesPackage = rulesPackage,
            fingerprint = compiledState.fingerprint,
            configContainer = compiledState.configContainer,
            systemContainer = compiledState.systemContainer,
            ruleContainer = compiledState.ruleContainer,
            derivedFeatures = compiledState.derivedFeatures,
            serviceRuntime = serviceRuntime
          )

          closeUnusedServiceRegistries(sta.serviceRuntime.registry, registry)

        case None =>
          loadSkip.incr()
      }
      loadSuccess.incr()
    } catch {
      case t: Throwable =>
        logger.warning(t.getMessage + "\n" + getStackTrace(t))
        loadFailure.incr()
        throw t
    } finally {
      state.dump()
    }
  }

  def closeUnusedServiceAfter: Duration = Duration.fromSeconds(30)

  protected def closeUnusedServiceRegistries(
    oldReg: Seq[ServiceRegistry[Any]],
    newReg: Seq[ServiceRegistry[Any]]
  ): Unit = {
    (oldReg.toSet diff newReg.toSet) foreach { reg =>
      val when = Time.fromMilliseconds(runtime.clock.nowMillis).plus(closeUnusedServiceAfter)
      runtime.timer.schedule(when) {
        Future.Unit flatMap { _ =>
          logger.info(s"closing unused ${reg.name}: ${reg.getClass.getName}")
          reg.close()
        } onFailure { t => logger.warning(s"failed to close ${reg.name}: $t") }
      }
    }
  }

  val success: Promise[Unit] = new Promise[Unit]

  val ready: Future[Var[Unit] with Extractable[Unit]] = scheduler(
    schedule.copy(
      loader = new Loader[Unit] {
        override val name: String = "botmaker"

        override def load(): Future[Unit] = {
          Future.Unit flatMap { _ =>
            schedule.loader.load()
          } map { rulesPackage =>
            BotMaker.this.load(rulesPackage)
            success.updateIfEmpty(Return.Unit)
          } handle {
            case e: Throwable =>
              success.updateIfEmpty(Throw(e))
              stateTransitionFailure.incr()
              if (state.isEmpty) {
                initStateFailure.incr()
              }
          } unit
        }
      }
    )
  ) interruptible

  override def close(deadline: Time): Future[Unit] = {
    scheduler.close(deadline)
  }

  def checkEvent[T <: EventOutputCollector](
    eventType: String,
    inputFeatures: InputProvider,
    outputCollector: T
  ): Future[T] = {
    Stat.timeFuture(eventStats.stat(eventType, "latency")) {

      eventStats.counter(eventType, "requests").incr()

      val sta = state
      val preOutput = new PreEventOutputCollector
      val cache: ExprCache = ExprCache()

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
            ) onFailure { t: Throwable =>
              exceptionCounter(eventType, t, "exceptions", "post").incr()
            } map { _ =>
              eventStats.counter(eventType, "successes").incr()
              outputCollector
            }
          }
      }
    }
  }

  protected def exceptionCounter(
    eventType: String,
    ex: Throwable,
    nameComponents: String*
  ): Counter = {
    val (nameType, verbosity) = classifyException(ex) match {
      case Interruption => ("interrupted", Verbosity.Default)
      case Ignorable => ("ignorable", Verbosity.Debug)
      case _ => ("unknown", Verbosity.Default)
    }
    val finalNames: Seq[String] =
      Seq(eventType) ++ nameComponents ++ Seq(nameType, ex.getClass.getSimpleName)
    eventStats.counter(verbosity, finalNames: _*)
  }

  def processEvent(
    eventType: String,
    inputFeatures: PartialFunction[String, AnyRef],
    outputCollector: OutputCollector
  ): Future[Unit] = {
    val sta = state

    if (sta.ruleContainer.isEventDefined(eventType)) {
      val cache: ExprCache = ExprCache()

      sta.checkEvent(
        eventType,
        cache,
        inputFeatures,
        outputCollector,
        PreEventOutput.empty,
        this,
        this,
        this,
        this
      )
    } else {
      Future.Done
    }
  }

  def runExpression(
    eventType: String,
    expr: String
  )(
    inputFeatures: InputProvider
  ): Future[(Type, AnyRef)] = {
    runExpression(eventType, expr, OutputCollector.empty)(inputFeatures)
  }

  def runExpression(
    eventType: String,
    expr: String,
    outputCollector: OutputCollector
  )(
    inputFeatures: InputProvider
  ): Future[(Type, AnyRef)] = {
    if (eventType == BotMaker.systemEvents.configEvent) {
      val astNode = state.configContainer.compiler
        .createContext()
        .compile(expr)
      val extractor: Extractor[Any] = astNode.toExtractor.asInstanceOf[Extractor[Any]]

      val configs = ct.runtimeClass.newInstance()
      val context = Context.builder.build(configs)
      context.run(extractor) map { r => (astNode.getReturnType, r) }
    } else if (eventType.startsWith(BotMaker.systemEventPrefix)) {
      val compilerContext =
        state.systemContainer.compiler
          .createContextBuilder()
          .setDerivedFeatures(state.derivedFeatures)
          .setInputFeatureNamespace(eventType)
          .enableContextScope(false)
          .build()
      val astNode = compilerContext.compile(expr)
      val extractor: Extractor[RT] = astNode.toExtractor
      val eventContext = EventContext[RT](
        inputRuntime = state.serviceRuntime,
        eventName = eventType,
        inputFeaturesProvider = inputFeatures,
        outputCollector = outputCollector
      )

      val context = RuleContext[RT](
        eventContext = eventContext,
        ruleId = -1,
        eventType = eventType,
        name = "runExpression",
        expr = expr
      )

      context.run(extractor) map { r => (astNode.getReturnType, r) }
    } else {
      val compilerContext =
        state.ruleContainer.compiler
          .createContextBuilder()
          .setDerivedFeatures(state.derivedFeatures)
          .setInputFeatureNamespace(eventType)
          .enableContextScope(false)
          .build()
      val astNode = compilerContext.compile(expr)
      val extractor: Extractor[RT] = astNode.toExtractor
      val eventContext = EventContext[RT](
        inputRuntime = state.serviceRuntime,
        eventName = eventType,
        inputFeaturesProvider = inputFeatures,
        outputCollector = outputCollector
      )

      val context = RuleContext[RT](
        eventContext = eventContext,
        ruleId = -1,
        eventType = eventType,
        name = "runExpression",
        expr = expr
      )

      context.run(extractor) map { r => (astNode.getReturnType, r) }
    }
  }

  def checkSyntax(
    eventType: String,
    expr: String,
    returnType: String,
    isDerivedFeature: Boolean
  ): Unit = {
    if (isDerivedFeature) {
      val systemCompilerContext: CompilerContext[RT] =
        state.systemContainer.compiler
          .createContextBuilder()
          .setDerivedFeatures(state.derivedFeatures)
          .enableContextScope(false)
          .build()
      val regularCompilerContext: CompilerContext[RT] =
        state.ruleContainer.compiler
          .createContextBuilder()
          .setDerivedFeatures(state.derivedFeatures)
          .enableContextScope(false)
          .build()
      val trySys: Try[ASTNode[RT]] = Try(systemCompilerContext.compileWithParams(expr))
      val tryReg: Try[ASTNode[RT]] = Try(regularCompilerContext.compileWithParams(expr))

      (trySys, tryReg) match {
        case (Throw(_), Throw(reg)) => throw reg
        case _ => ()
      }
    } else {
      val compilerContext: CompilerContext[_] =
        if (eventType == BotMaker.systemEvents.configEvent) {
          state.configContainer.compiler
            .createContextBuilder()
            .setDerivedFeatures(DerivedFeatureProvider.EMPTY)
            .setInputFeatureNamespace(eventType)
            .build()

        } else if (eventType.startsWith(BotMaker.systemEventPrefix)) {
          state.systemContainer.compiler
            .createContextBuilder()
            .setDerivedFeatures(state.derivedFeatures)
            .setInputFeatureNamespace(eventType)
            .enableContextScope(false)
            .build()
        } else {
          state.ruleContainer.compiler
            .createContextBuilder()
            .setDerivedFeatures(state.derivedFeatures)
            .setInputFeatureNamespace(eventType)
            .enableContextScope(false)
            .build()
        }

      val astNode: ASTNode[_] = compilerContext.compile(expr)

      if (returnType == "Boolean" && Type.isDivergentTo(Type.BOOLEAN, astNode.getReturnType)) {
        throw new SemanticCheckFailure(
          "Boolean expression expects return type of %s but %s received.".format(
            Type.BOOLEAN,
            astNode.getReturnType
          )
        )
      }
    }
  }

  def checkPackage(rulesPackage: BotMakerRulesPackage): Future[Unit] = {
    state.compile(rulesPackage) match {
      case Some(compiledState)
          if compiledState.ruleContainer.hasCompilingFailure ||
            compiledState.systemContainer.hasCompilingFailure =>
        val failedRules = (compiledState.systemContainer.failedRules ++
          compiledState.ruleContainer.failedRules) map { r =>
          (r.id, r.name, r.throwable.getMessage)
        }
        Future.exception(
          new SemanticCheckFailure(
            s"RulesPackage contains compile error. Failed rules: ${failedRules.mkString(", ")}"
          )
        )
      case Some(compiledState) =>
        val sta = State(
          rulesPackage = compiledState.rulesPackage,
          fingerprint = compiledState.fingerprint,
          configContainer = compiledState.configContainer,
          systemContainer = compiledState.systemContainer,
          ruleContainer = compiledState.ruleContainer,
          derivedFeatures = compiledState.derivedFeatures,
          serviceRuntime = this.state.serviceRuntime
        )
        sta.validatePackage
      case _ =>
        Future.Done
    }
  }

  def reportStat(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    statName: String,
    value: Long
  ): Unit = {
    state.checkSysEvent(
      BotMaker.systemEvents.reportStat,
      ExprCache.empty,
      inputFeatures = {
        case "namespace" => namespace
        case "eventType" => eventType
        case "ruleId" => Long.box(ruleId)
        case "statName" => statName
        case "value" => Long.box(value)
      },
      OutputCollector.empty
    )
  }

  def reportFailure(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    category: String,
    t: Throwable
  ): Unit = {
    state.checkSysEvent(
      BotMaker.systemEvents.reportFailure,
      ExprCache.empty,
      inputFeatures = {
        case "namespace" => namespace
        case "eventType" => eventType
        case "ruleId" => Long.box(ruleId)
        case "category" => category
        case "exceptionType" => t.getClass.getName
        case "exceptionMessage" => t.getMessage
      },
      OutputCollector.empty
    )
  }

  def reportFunctionEvaluation(
    functionName: String,
    actionLevel: ActionLevel,
    args: Seq[_],
    ruleId: Long
  ): Unit = {
    if (isUserImpacting(actionLevel)) {
      userImpactByRuleStats
        .counter(ruleId.toString, "action_level", actionLevel.toString, "function", functionName)
        .incr()
      userImpactByRuleStats
        .counter(ruleId.toString, "action_level", actionLevel.toString, "action_count")
        .incr()
    }
  }

  private[this] def isUserImpacting(actionLevel: ActionLevel): Boolean =
    actionLevel == ActionLevel.PROD_AGENT_WORKFLOW_ACTION ||
      actionLevel == ActionLevel.PROD_OTHER_ACTION ||
      actionLevel == ActionLevel.PROD_TANGIBLE_IMPACT_ACTION ||
      actionLevel == ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION

  override def applyActionRateLimit[V](
    context: Context[RT],
    functionName: String,
    actionLevel: ActionLevel,
    id: Long,
    actionLimits: collection.Map[String, Seq[ActionLimit]],
    func: () => Future[V],
    args: Seq[_]
  ): Future[V] = func.apply()
}

case class UsageInfo(ruleIds: Set[Long], derivedFeatureNames: Set[String])
