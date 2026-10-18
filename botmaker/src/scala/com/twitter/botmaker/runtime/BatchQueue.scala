package com.twitter.botmaker.runtime

import java.util
import scala.collection.JavaConverters._
import scala.collection.mutable
import scala.reflect.ClassTag
import scala.runtime.BoxedUnit
import com.google.common.base.Strings
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.common.AsyncRateLimiter
import com.twitter.botmaker.common.BatchDispatcher
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.types.TypeCompiler
import com.twitter.botmaker.compiler.types.TypeContext
import com.twitter.botmaker.compiler.tuples.Tuple
import com.twitter.botmaker.function.feature.OutputFeature
import com.twitter.botmaker.function.FunctionNode
import com.twitter.botmaker.function.GeneralNode
import com.twitter.botmaker.function.thrift.ThriftOperator
import com.twitter.botmaker.rule.EventProcessor
import com.twitter.botmaker.rule.OutputCollector
import com.twitter.botmaker.runtime.config.BatchQueueConfigs
import com.twitter.botmaker.runtime.thriftjava.BatchQueueConfig
import com.twitter.botmaker.runtime.thriftjava.BotMakerEventTypeConfig
import com.twitter.botmaker.runtime.thriftjava.ThriftFieldConfig
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.common.util.Clock
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util.Future
import com.twitter.util.Promise
import com.twitter.util.Return
import com.twitter.util.Throw
import com.twitter.util.Time

case class OneWayBatchQueueRegistry(
  val config: BatchQueueConfig,
  private val queue: BatchDispatcher[AnyRef])
    extends ServiceRegistry[BatchQueueConfig] {

  override val name: String = config.eventType

  override def close(deadline: Time): Future[Unit] = Future.Unit

  def add(requests: util.List[AnyRef]): Unit = {
    add(requests.asScala)
  }

  def add(requests: Seq[AnyRef]): Future[BoxedUnit] = {
    queue.add(requests)
    Future.value(BoxedUnit.UNIT)
  }
}

case class UnitBatchQueueRegistry(
  val config: BatchQueueConfig,
  private val queue: BatchDispatcher[(Promise[Unit], AnyRef)])
    extends ServiceRegistry[BatchQueueConfig] {

  override val name: String = config.eventType

  override def close(deadline: Time): Future[Unit] = Future.Unit

  def add(requests: util.List[AnyRef]): Future[Unit] = {
    add(requests.asScala)
  }

  def add(requests: Seq[AnyRef]): Future[Unit] = {
    val promises = requests.map(new Promise[Unit] -> _)
    queue.add(promises)
    Future.join(promises.map(_._1))
  }
}

case class BatchQueueRegistry(
  val config: BatchQueueConfig,
  private val queue: BatchDispatcher[(Promise[AnyRef], AnyRef)])
    extends ServiceRegistry[BatchQueueConfig] {

  override val name: String = config.eventType

  override def close(deadline: Time): Future[Unit] = Future.Unit

  def add(requests: util.List[AnyRef]): Future[util.List[AnyRef]] = {
    add(requests.asScala).map(_.asJava)
  }

  def add(requests: Seq[AnyRef]): Future[Seq[AnyRef]] = {
    val promises = requests.map(new Promise[AnyRef] -> _)
    queue.add(promises)
    Future.collect(promises.map(_._1))
  }
}

trait BatchQueueContainer extends ServiceContainer with EventProcessor {
  container =>

  override type _CONFIGS_TYPE <: BatchQueueConfigs

  private def isReusable(config: BatchQueueConfig) =
    isReusable[BatchQueueConfig, OneWayBatchQueueRegistry](config.eventType, config) ||
      isReusable[BatchQueueConfig, UnitBatchQueueRegistry](config.eventType, config) ||
      isReusable[BatchQueueConfig, BatchQueueRegistry](config.eventType, config)

  def oneWayBatchQueues(config: BatchQueueConfig): OneWayBatchQueueRegistry = {
    serviceRegistry[OneWayBatchQueueRegistry](config.eventType).get
  }

  def unitBatchQueues(config: BatchQueueConfig): UnitBatchQueueRegistry = {
    serviceRegistry[UnitBatchQueueRegistry](config.eventType).get
  }

  def batchQueues(config: BatchQueueConfig): BatchQueueRegistry = {
    serviceRegistry[BatchQueueRegistry](config.eventType).get
  }

  private def buildOneWay(config: BatchQueueConfig, scoped: StatsReceiver) = {
    val queue = new BatchDispatcher[AnyRef] {

      override def queueSize: Int = config.queueSize
      override def batchSize: Int = config.batchSize
      override def concurrentSize: Int = config.concurrentSize

      override def rateLimiter: AsyncRateLimiter = AsyncRateLimiter.noop
      override def statsReceiver: StatsReceiver = scoped.scope(config.eventType)
      override def clock: Clock = runtime.clock

      override def dispatch(batch: Seq[AnyRef]): Future[Unit] = {
        val inputBatch = batch.toList.asJava
        processEvent(
          config.eventType,
          inputFeatures = {
            case "batch" =>
              inputBatch
          }
        )
      }
    }

    OneWayBatchQueueRegistry(config, queue)
  }

  private def buildUnit(config: BatchQueueConfig, scoped: StatsReceiver) = {
    val queue = new BatchDispatcher[(Promise[Unit], AnyRef)] {

      override def queueSize: Int = config.queueSize
      override def batchSize: Int = config.batchSize
      override def concurrentSize: Int = config.concurrentSize

      override def rateLimiter: AsyncRateLimiter = AsyncRateLimiter.noop
      override def statsReceiver: StatsReceiver = scoped.scope(config.eventType)
      override def clock: Clock = runtime.clock

      override def dispatch(batch: Seq[(Promise[Unit], AnyRef)]): Future[Unit] = {
        val inputBatch = batch.map(_._2).toList.asJava
        Future.Unit flatMap { _ =>
          processEvent(
            config.eventType,
            inputFeatures = {
              case "batch" =>
                inputBatch
            }
          )
        } onSuccess { _ =>
          batch.map(_._1) foreach { _.setDone() }
        } onFailure { t: Throwable =>
          batch.map(_._1) foreach {
            _.setException(t)
          }
        }
      }
    }

    UnitBatchQueueRegistry(config, queue)
  }

  private def buildReturn(config: BatchQueueConfig, scoped: StatsReceiver) = {
    val queue = new BatchDispatcher[(Promise[AnyRef], AnyRef)] {

      override def queueSize: Int = config.queueSize
      override def batchSize: Int = config.batchSize
      override def concurrentSize: Int = config.concurrentSize

      override def rateLimiter: AsyncRateLimiter = AsyncRateLimiter.noop
      override def statsReceiver: StatsReceiver = scoped.scope(config.eventType)
      override def clock: Clock = runtime.clock

      override def dispatch(batch: Seq[(Promise[AnyRef], AnyRef)]): Future[Unit] = {
        val inputBatch = batch.map(_._2).toList.asJava

        val outputCollector = new OutputCollector {
          override def addOutputFeature(
            namespace: AnyRef,
            eventType: String,
            ruleId: Long,
            fn: String,
            ft: Type,
            fv: AnyRef
          ) = {
            fv match {
              case x: Tuple
                  if namespace == OutputFeature.NAMESPACE && fn == "result" && x.size == 2 =>
                batch.filter(_._2 == x.get(0)) foreach {
                  _._1.updateIfEmpty(Return(x.get(1)))
                }
              case _ =>
                ()
            }
          }
        }

        Future.Unit flatMap { _ =>
          processEvent(
            config.eventType,
            inputFeatures = {
              case "batch" =>
                inputBatch
            },
            outputCollector = outputCollector
          )
        } onSuccess { _ =>
          val t = new Throwable("cancelled")
          batch.map(_._1) foreach { _.updateIfEmpty(Throw(t)) }
        } onFailure { t: Throwable =>
          batch.map(_._1) foreach {
            _.updateIfEmpty(Throw(t))
          }
        }
      }
    }

    BatchQueueRegistry(config, queue)
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val batchQueues = configs.getBatchQueues.map {
      case config if isReusable(config) =>
        logger.info(s"reuse batch queue ${config.eventType}")
        serviceRegistry[OneWayBatchQueueRegistry](config.eventType) orElse {
          serviceRegistry[UnitBatchQueueRegistry](config.eventType)
        } orElse {
          serviceRegistry[BatchQueueRegistry](config.eventType)
        } get

      case config if Strings.isNullOrEmpty(config.returnType) =>
        logger.info(s"create oneway batch queue ${config.eventType}")
        buildOneWay(config, scoped.scope("BatchQueue"))

      case config if config.returnType == "Unit" =>
        logger.info(s"create unit batch queue ${config.eventType}")
        buildUnit(config, scoped.scope("BatchQueue"))

      case config =>
        logger.info(s"create batch queue ${config.eventType}")
        buildReturn(config, scoped.scope("BatchQueue"))
    }
    batchQueues ++ super.build(configs, scoped)
  }
}

case class OneWayBatchQueueOperator(config: BatchQueueConfig) {

  private type Thrift = TwitterRuntime.Thrift[_, _]

  private val ns = config.eventType + "."
  private val addFuncName = ns + "Queue"

  def toCompile(typeContext: TypeContext): List[CompilerUnit] = {

    val itemType: Type = TypeCompiler.getType(config.itemType, typeContext)

    val addFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.of[Type](),
        itemType,
        Type.UNIT
      )

      override def funcName = addFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"add requests to BatchQueue ${config.eventType}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode[BatchQueueContainer](addFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(
            context: Context[BatchQueueContainer],
            args: util.List[AnyRef]
          ) = {
            val batchQueue = context.getRuntime.oneWayBatchQueues(config)
            batchQueue.add(args)
            unit
          }
        }
      }
    }

    List(addFunc)
  }
}

case class UnitBatchQueueOperator(config: BatchQueueConfig) {

  private type Thrift = TwitterRuntime.Thrift[_, _]

  private val ns = config.eventType + "."
  private val addFuncName = ns + "Queue"

  def toCompile(typeContext: TypeContext): List[CompilerUnit] = {

    val itemType: Type = TypeCompiler.getType(config.itemType, typeContext)

    val addFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.of[Type](),
        itemType,
        Type.UNIT
      )

      override def funcName = addFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"add requests to BatchQueue ${config.eventType}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[BatchQueueContainer](addFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[BatchQueueContainer],
            args: util.List[AnyRef]
          ) = {
            val batchQueue = context.getRuntime.unitBatchQueues(config)
            batchQueue.add(args) map { _ => unit }
          }
        }
      }
    }

    List(addFunc)
  }
}

case class BatchQueueOperator(config: BatchQueueConfig) {

  private type Thrift = TwitterRuntime.Thrift[_, _]

  private val ns = config.eventType + "."
  private val addFuncName = ns + "Queue"
  private val addBatchFuncName = ns + "Batch"

  def toCompile(typeContext: TypeContext): List[CompilerUnit] = {

    val itemType: Type = TypeCompiler.getType(config.itemType, typeContext)
    val returnType: Type = TypeCompiler.getType(config.returnType, typeContext)

    val addFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.of(itemType),
        returnType
      )

      override def funcName = addFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"add requests to BatchQueue ${config.eventType}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[BatchQueueContainer](addFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[BatchQueueContainer],
            args: util.List[AnyRef]
          ) = {
            val batchQueue = context.getRuntime.batchQueues(config)
            batchQueue.add(args).map(_.get(0))
          }
        }
      }
    }

    val addBatchFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.of[Type](),
        itemType,
        Type.listOf(returnType)
      )

      override def funcName = addBatchFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"add requests to BatchQueue ${config.eventType}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[BatchQueueContainer](addFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[BatchQueueContainer],
            args: util.List[AnyRef]
          ) = {
            val batchQueue = context.getRuntime.batchQueues(config)
            batchQueue.add(args)
          }
        }
      }
    }

    List(addFunc, addBatchFunc)
  }
}
