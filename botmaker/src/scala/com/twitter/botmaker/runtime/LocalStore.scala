package com.twitter.botmaker.runtime

import java.util
import java.util.concurrent.TimeUnit
import scala.collection.JavaConverters._
import com.google.common.cache._
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerDoc
import com.twitter.botmaker.compiler.CompilerUnit
import com.twitter.botmaker.compiler.Fingerprint
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.types.TypeCompiler
import com.twitter.botmaker.compiler.types.TypeContext
import com.twitter.botmaker.compiler.tuples.Tuple
import com.twitter.botmaker.function.FunctionNode
import com.twitter.botmaker.function.FunctionNode0
import com.twitter.botmaker.runtime.config.LocalStoreConfigs
import com.twitter.botmaker.runtime.thriftjava.LocalStoreConfig
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.finagle.stats.Gauge
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util.Future
import com.twitter.util.Time

import java.util.concurrent.ConcurrentMap

case class LocalStoreRegistry(
  config: LocalStoreConfig,
  private val cache: Cache[Tuple, AnyRef],
  gauge: Gauge)
    extends ServiceRegistry[LocalStoreConfig] {

  override val name: String = config.funcName

  override def close(deadline: Time): Future[Unit] = Future.Unit

  def get(key: Tuple): AnyRef = cache.getIfPresent(key)

  def put(key: Tuple, value: AnyRef): Unit = {
    cache.put(key, value)
  }

  def asMap: ConcurrentMap[Tuple, AnyRef] = cache.asMap
}

case class LocalVarRegistry(config: LocalStoreConfig, private var value: AnyRef)
    extends ServiceRegistry[LocalStoreConfig] {

  override val name: String = config.funcName

  override def close(deadline: Time): Future[Unit] = Future.Unit

  def get: AnyRef = value

  def set(x: AnyRef): Unit = {
    value = x
  }
}

trait LocalStoreFactory extends TwitterRuntime {

  def build(config: LocalStoreConfig, scoped: StatsReceiver): (Cache[Tuple, AnyRef], Gauge) = {

    val sizeRemoval = scoped.counter(config.funcName, "cache_size_removal")
    val expiredRemoval = scoped.counter(config.funcName, "cache_expired_removal")
    val otherRemoval = scoped.counter(config.funcName, "cache_other_removal")
    val cache: Cache[Tuple, AnyRef] = CacheBuilder.newBuilder
      .maximumSize(config.maximumCacheItems)
      .expireAfterAccess(config.cacheExpirationMillis, TimeUnit.MILLISECONDS)
      .removalListener(new RemovalListener[Tuple, AnyRef] {
        override def onRemoval(removalNotification: RemovalNotification[Tuple, AnyRef]): Unit = {
          removalNotification.getCause match {
            case RemovalCause.SIZE => sizeRemoval.incr()
            case RemovalCause.EXPIRED => expiredRemoval.incr()
            case _ => otherRemoval.incr()
          }
        }
      })
      .build[Tuple, AnyRef]

    val sizeGauge: Gauge = scoped.addGauge(config.funcName, "cache_size") {
      cache.size
    }

    (cache, sizeGauge)
  }
}

trait LocalStoreContainer extends ServiceContainer with LocalStoreFactory {
  container =>

  override type _CONFIGS_TYPE <: LocalStoreConfigs

  private def isReusable(config: LocalStoreConfig) =
    isReusable[LocalStoreConfig, LocalStoreRegistry](config.funcName, config) ||
      isReusable[LocalStoreConfig, LocalVarRegistry](config.funcName, config)

  def localStores(config: LocalStoreConfig): LocalStoreRegistry = {
    serviceRegistry[LocalStoreRegistry](config.funcName).get
  }

  def localVars(config: LocalStoreConfig): LocalVarRegistry = {
    serviceRegistry[LocalVarRegistry](config.funcName).get
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val localStores = configs.getLocalStores.map {

      case config if isReusable(config) =>
        logger.info(s"reuse local counter ${config.funcName}")
        serviceRegistry[LocalStoreRegistry](config.funcName) orElse {
          serviceRegistry[LocalVarRegistry](config.funcName)
        } get
      case config if config.keyTypes.size > 0 =>
        logger.info(s"create local counter ${config.funcName}")
        val (cache, gauge) = build(config, scoped.scope("LocalStore"))
        LocalStoreRegistry(config, cache, gauge)
      case config if config.keyTypes.size == 0 =>
        logger.info(s"create local var ${config.funcName}")
        LocalVarRegistry(config, null)
    }
    localStores ++ super.build(configs, scoped)
  }
}

case class LocalStoreOperator(config: LocalStoreConfig) {

  private type Thrift = TwitterRuntime.Thrift[_, _]

  private val ns = "local."
  private val setFuncName = ns + "Set" + config.funcName
  private val getFuncName = ns + "Get" + config.funcName
  private val mapFuncName = ns + config.funcName

  val name: String = config.funcName

  def toCompile(typeContext: TypeContext): List[CompilerUnit] = {

    val keyTypes = config.keyTypes.asScala map { a => TypeCompiler.getType(a, typeContext) } toList

    val valueType = TypeCompiler.getType(config.valueType, typeContext)

    val setFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf((keyTypes :+ valueType).toIterable.asJava),
        Type.UNIT
      )

      override def funcName = setFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"set value by key in local store ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode[LocalStoreContainer](setFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(
            context: Context[LocalStoreContainer],
            args: util.List[AnyRef]
          ) = {
            val key = Tuple.of(args.asScala.take(keyTypes.size).toArray)
            val store = context.getRuntime.localStores(config)
            store.put(key, args.get(args.size() - 1))
            unit
          }
        }
      }
    }

    val getFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf(keyTypes.toIterable.asJava),
        ImmutableList.of(valueType),
        valueType
      )

      override def funcName = getFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"get value by key in local store ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode[LocalStoreContainer](getFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(
            context: Context[LocalStoreContainer],
            args: util.List[AnyRef]
          ) = {
            val key = Tuple.of(args.asScala.take(keyTypes.size).toArray)
            val store = context.getRuntime.localStores(config)
            val value = store.get(key)
            if (value != null) {
              value
            } else if (args.asScala.lengthCompare(keyTypes.size) > 0) {
              args.get(args.asScala.length - 1)
            } else {
              null
            }
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
        s"get local store map ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode0[LocalStoreContainer](getFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(context: Context[LocalStoreContainer]) = {
            val store = context.getRuntime.localStores(config)
            store.asMap
          }
        }
      }
    }

    List(setFunc, getFunc, mapFunc)
  }
}

case class LocalVarOperator(config: LocalStoreConfig) {

  private type Thrift = TwitterRuntime.Thrift[_, _]

  private val ns = "local."
  private val setFuncName = ns + "Set" + config.funcName
  private val getFuncName = ns + "Get" + config.funcName

  val name: String = config.funcName

  def toCompile(typeContext: TypeContext): List[CompilerUnit] = {

    val valueType = TypeCompiler.getType(config.valueType, typeContext)

    val setFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.of(valueType),
        Type.UNIT
      )

      override def funcName = setFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"set value for local var ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode[LocalStoreContainer](setFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(
            context: Context[LocalStoreContainer],
            args: util.List[AnyRef]
          ) = {
            context.getRuntime.localVars(config).set(args.get(0))
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
        s"get local var ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode[LocalStoreContainer](getFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(
            context: Context[LocalStoreContainer],
            args: util.List[AnyRef]
          ) = {
            context.getRuntime.localVars(config).get
          }
        }
      }
    }

    List(setFunc, getFunc)
  }
}
