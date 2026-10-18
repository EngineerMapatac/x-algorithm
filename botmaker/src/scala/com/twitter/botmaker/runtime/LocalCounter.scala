package com.twitter.botmaker.runtime

import java.util
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicLong
import scala.collection.JavaConverters._
import com.google.common.cache._
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.types.TypeCompiler
import com.twitter.botmaker.compiler.types.TypeContext
import com.twitter.botmaker.compiler.tuples.Tuple
import com.twitter.botmaker.function.FunctionNode
import com.twitter.botmaker.function.FunctionNode0
import com.twitter.botmaker.runtime.config.LocalCounterConfigs
import com.twitter.botmaker.runtime.thriftjava.LocalCounterConfig
import com.twitter.finagle.stats.Gauge
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util.Future
import com.twitter.util.Time

case class LocalCounterStoreRegistry(
  config: LocalCounterConfig,
  cache: LoadingCache[Tuple, AtomicLong],
  gauge: Gauge)
    extends ServiceRegistry[LocalCounterConfig] {

  override val name: String = config.funcName

  override def close(deadline: Time): Future[Unit] = Future.Unit
}

case class LocalCounterRegistry(config: LocalCounterConfig, value: AtomicLong)
    extends ServiceRegistry[LocalCounterConfig] {

  override val name: String = config.funcName

  override def close(deadline: Time): Future[Unit] = Future.Unit
}

trait LocalCounterFactory extends TwitterRuntime {

  def build(
    config: LocalCounterConfig,
    scoped: StatsReceiver
  ): (LoadingCache[Tuple, AtomicLong], Gauge) = {

    val sizeRemoval = scoped.counter(config.funcName, "cache_size_removal")
    val expiredRemoval = scoped.counter(config.funcName, "cache_expired_removal")
    val otherRemoval = scoped.counter(config.funcName, "cache_other_removal")
    val cache: LoadingCache[Tuple, AtomicLong] = CacheBuilder.newBuilder
      .maximumSize(config.maximumCacheItems)
      .expireAfterAccess(config.cacheExpirationMillis, TimeUnit.MILLISECONDS)
      .removalListener(new RemovalListener[Tuple, AtomicLong] {
        override def onRemoval(
          removalNotification: RemovalNotification[Tuple, AtomicLong]
        ): Unit = {
          removalNotification.getCause match {
            case RemovalCause.SIZE => sizeRemoval.incr()
            case RemovalCause.EXPIRED => expiredRemoval.incr()
            case _ => otherRemoval.incr()
          }
        }
      })
      .asInstanceOf[CacheBuilder[Tuple, AtomicLong]]
      .build(new CacheLoader[Tuple, AtomicLong] {
        override def load(k: Tuple) = new AtomicLong
      })

    val sizeGauge: Gauge = scoped.addGauge(config.funcName, "cache_size") {
      cache.size
    }

    (cache, sizeGauge)
  }
}

trait LocalCounterContainer extends ServiceContainer with LocalCounterFactory {
  container =>

  override type _CONFIGS_TYPE <: LocalCounterConfigs

  private def isReusable(config: LocalCounterConfig) =
    isReusable[LocalCounterConfig, LocalCounterRegistry](config.funcName, config) ||
      isReusable[LocalCounterConfig, LocalCounterStoreRegistry](config.funcName, config)

  def localCounters(config: LocalCounterConfig): LocalCounterRegistry = {
    serviceRegistry[LocalCounterRegistry](config.funcName).get
  }

  def localCounterStores(config: LocalCounterConfig): LoadingCache[Tuple, AtomicLong] = {
    serviceRegistry[LocalCounterStoreRegistry](config.funcName).get.cache
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val localCounters = configs.getLocalCounters.map {
      case config if isReusable(config) =>
        logger.info(s"reuse local counter ${config.funcName}")
        serviceRegistry[LocalCounterRegistry](config.funcName) orElse {
          serviceRegistry[LocalCounterStoreRegistry](config.funcName)
        } get
      case config if config.keyTypes.size > 0 =>
        logger.info(s"create local counter store ${config.funcName}")
        val (cache, gauge) = build(config, scoped.scope("LocalCounterStore"))
        LocalCounterStoreRegistry(config, cache, gauge)
      case config if config.keyTypes.size == 0 =>
        logger.info(s"create local counter ${config.funcName}")
        LocalCounterRegistry(config, new AtomicLong())
    }
    localCounters ++ super.build(configs, scoped)
  }
}

case class LocalCounterStoreOperator(config: LocalCounterConfig) {

  private type Thrift = TwitterRuntime.Thrift[_, _]

  private val ns = "local."
  private val incrFuncName = ns + "Incr" + config.funcName
  private val setFuncName = ns + "Set" + config.funcName
  private val getFuncName = ns + "Get" + config.funcName
  private val mapFuncName = ns + config.funcName

  val name: String = config.funcName

  def toCompile(typeContext: TypeContext): List[CompilerUnit] = {

    val keyTypes = config.keyTypes.asScala map { a => TypeCompiler.getType(a, typeContext) } toList

    val valueType = Type.LONG

    val incrFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf(keyTypes.toIterable.asJava),
        ImmutableList.of(valueType),
        valueType
      )

      override def funcName = incrFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"increment value by key in local counter ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode[LocalCounterContainer](incrFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(
            context: Context[LocalCounterContainer],
            args: util.List[AnyRef]
          ) = {
            val key = Tuple.of(args.asScala.take(keyTypes.size).toArray)
            val delta: java.lang.Long = if (args.size > keyTypes.size) {
              args.get(args.size - 1).asInstanceOf[java.lang.Long]
            } else {
              Long.box(1L)
            }
            val cache = context.getRuntime.localCounterStores(config)
            val counter = cache.get(key)
            counter.addAndGet(delta).asInstanceOf[java.lang.Long]
          }
        }
      }
    }

    val setFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf((keyTypes :+ valueType).toIterable.asJava),
        Type.UNIT
      )

      override def funcName = setFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"set value by key in local counter ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode[LocalCounterContainer](setFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(
            context: Context[LocalCounterContainer],
            args: util.List[AnyRef]
          ) = {
            val key = Tuple.of(args.asScala.take(keyTypes.size).toArray)
            val cache = context.getRuntime.localCounterStores(config)
            val counter = cache.get(key)
            counter.set(args.get(args.size - 1).asInstanceOf[java.lang.Long])
            unit
          }
        }
      }
    }

    val getFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf(keyTypes.toIterable.asJava),
        valueType
      )

      override def funcName = getFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"get value by key in local counter ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode[LocalCounterContainer](getFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(
            context: Context[LocalCounterContainer],
            args: util.List[AnyRef]
          ) = {
            val key = Tuple.of(args.toArray)
            val cache = context.getRuntime.localCounterStores(config)
            val counter = cache.get(key)
            counter.get.asInstanceOf[java.lang.Long]
          }
        }
      }
    }

    val mapFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.of[Type](),
        Type.mapOf(Type.tupleOf(keyTypes: _*), valueType)
      )

      override def funcName = mapFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"get local counters map ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode0[LocalCounterContainer](getFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(context: Context[LocalCounterContainer]) = {
            val cache = context.getRuntime.localCounterStores(config)
            cache.asMap.entrySet.asScala.map(e => e.getKey -> e.getValue.get).toMap.asJava
          }
        }
      }
    }

    List(incrFunc, setFunc, getFunc, mapFunc)
  }
}

case class LocalCounterOperator(config: LocalCounterConfig) {

  private type Thrift = TwitterRuntime.Thrift[_, _]

  private val ns = "local."
  private val incrFuncName = ns + "Incr" + config.funcName
  private val setFuncName = ns + "Set" + config.funcName
  private val getFuncName = ns + "Get" + config.funcName

  val name: String = config.funcName

  def toCompile: List[CompilerUnit] = {

    val valueType = Type.LONG

    val incrFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.of[Type](),
        ImmutableList.of(valueType),
        valueType
      )

      override def funcName = incrFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"increment value by key in local counter ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode[LocalCounterContainer](incrFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(
            context: Context[LocalCounterContainer],
            args: util.List[AnyRef]
          ) = {
            val delta: java.lang.Long = if (args.asScala.length > 0) {
              args.get(args.asScala.length - 1).asInstanceOf[java.lang.Long]
            } else {
              Long.box(1L)
            }
            val counter = context.getRuntime.localCounters(config)
            counter.value.addAndGet(delta).asInstanceOf[java.lang.Long]
          }
        }
      }
    }

    val setFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.of(valueType),
        Type.UNIT
      )

      override def funcName = setFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"set value by key in local counter ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode[LocalCounterContainer](setFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(
            context: Context[LocalCounterContainer],
            args: util.List[AnyRef]
          ) = {
            val counter = context.getRuntime.localCounters(config)
            counter.value.set(args.get(0).asInstanceOf[java.lang.Long])
            unit
          }
        }
      }
    }

    val getFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.of[Type],
        valueType
      )

      override def funcName = getFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"get value by key in local counter ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode[LocalCounterContainer](getFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(
            context: Context[LocalCounterContainer],
            args: util.List[AnyRef]
          ) = {
            val key = args.asScala.toList
            val counter = context.getRuntime.localCounters(config)
            counter.value.get.asInstanceOf[java.lang.Long]
          }
        }
      }
    }

    List(incrFunc, setFunc, getFunc)
  }
}
