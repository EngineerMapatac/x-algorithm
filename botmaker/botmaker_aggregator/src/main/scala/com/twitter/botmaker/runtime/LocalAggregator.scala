package com.twitter.botmaker.runtime

import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.types.TypeCompiler
import com.twitter.botmaker.compiler.types.TypeContext
import com.twitter.botmaker.compiler.tuples.Tuple
import com.twitter.botmaker.function.GeneralNode
import com.twitter.botmaker.rule.EventProcessor
import com.twitter.botmaker.runtime.config.AggregatorProcessor
import com.twitter.botmaker.runtime.config.AggregatorUnit
import com.twitter.botmaker.runtime.config.LocalAggregatorConfigs
import com.twitter.botmaker.runtime.thriftjava._
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util.Duration
import com.twitter.util.Future
import com.twitter.util.Time

import java.util
import scala.collection.JavaConverters._
import scala.collection.mutable.ArrayBuffer

case class LocalAggregatorRegistry(
  val config: LocalAggregatorConfig,
  aggregator: MonoidAggregator[_, _, _])
    extends ServiceRegistry[LocalAggregatorConfig] {

  override val name: String = config.eventType

  override def close(deadline: Time): Future[Unit] = {
    aggregator.close(deadline)
  }

  def add(key: AnyRef, amount: AnyRef): Future[Unit] = {
    aggregator.post(key, amount)
  }
}

trait LocalAggregatorContainer extends ServiceContainer with EventProcessor {
  container =>

  override type _CONFIGS_TYPE <: LocalAggregatorConfigs

  private def isReusable(config: LocalAggregatorConfig) =
    isReusable[LocalAggregatorConfig, LocalAggregatorRegistry](config.eventType, config)

  def localAggregators(config: LocalAggregatorConfig): LocalAggregatorRegistry = {
    serviceRegistry[LocalAggregatorRegistry](config.eventType).get
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val localAggregators = configs.getLocalAggregators.map {
      case actor if isReusable(actor.config) =>
        logger.info(s"reuse local aggregator ${actor.config.eventType}")
        serviceRegistry[LocalAggregatorRegistry](actor.config.eventType).get

      case actor =>
        logger.info(s"create local aggregator ${actor.config.eventType}")
        actor.registry(this)
    }

    localAggregators ++ super.build(configs, scoped)
  }
}

trait LocalAggregatorActor {

  def config: LocalAggregatorConfig

  def eventType: BotMakerEventTypeConfig

  def compilerUnits(tc: TypeContext): Seq[CompilerUnit]

  def registry(runtime: LocalAggregatorContainer): LocalAggregatorRegistry
}

object LocalAggregatorActor {

  def apply[I, M, O](
    au: AggregatorUnit[I, M, O],
    cfg: LocalAggregatorConfig,
    kt: String,
    vt: String,
    rt: String
  ): LocalAggregatorActor = {

    val isAggregatorProcessor = au.isInstanceOf[AggregatorProcessor]

    new LocalAggregatorActor {

      val config: LocalAggregatorConfig = cfg

      val eventType: BotMakerEventTypeConfig = new BotMakerEventTypeConfig()
        .setEventType(config.eventType)
        .setInputFeatures(
          ImmutableList.of(
            new ThriftFieldConfig()
              .setFieldName("stats")
              .setFieldId(2)
              .setFieldType(s"List<Pair<$kt,$rt>>")
          )
        )

      val op = LocalAggregatorOperator(config, kt, vt)

      override def compilerUnits(tc: TypeContext): Seq[CompilerUnit] = {
        op.toCompile(tc)
      }

      override def registry(runtime: LocalAggregatorContainer): LocalAggregatorRegistry = {
        val at = new MonoidAggregator[I, M, O](
          runtime,
          Duration.fromMilliseconds(config.flushIntervalMillis),
          config.flushSlices,
          config.maximumCacheItems,
          Duration.fromMilliseconds(config.cacheExpirationMillis)
        ) {

          override protected def zero: M = au.zero(config)

          override def plus(stat: M, amount: I): Unit = au.plus(config, stat, amount)

          override def flush(stream: Stream[Entry]): Future[Unit] = {
            val batches = stream
              .filter(s => au.filter(config, s.getValue))
              .sliding(config.flushBatchSize, config.flushBatchSize)

            def drain: Future[Unit] = {
              if (batches.hasNext) {
                val stats = ArrayBuffer[Tuple]()
                batches.next foreach { entry =>
                  val statValue = au.flush(config, entry.getValue)
                  stats += Tuple.of(entry.getKey, statValue)
                }

                val javaStats = stats.asJava
                var future = runtime.processEvent(
                  config.eventType,
                  inputFeatures = {
                    case "stats" =>
                      javaStats
                  }
                )

                future flatMap { _ => drain }
              } else {
                Future.Unit
              }
            }

            drain
          }
        }

        LocalAggregatorRegistry(config, at)
      }
    }
  }
}

case class LocalAggregatorOperator(
  config: LocalAggregatorConfig,
  keyTypeName: String,
  valueTypeName: String) {

  private val ns = config.eventType + "."
  private val funcName = ns + "Post"

  def toCompile(typeContext: TypeContext): Seq[CompilerUnit] = {
    val post = new CompilerUnit {

      private val keyType = TypeCompiler.getType(keyTypeName, typeContext)
      private val valueType = TypeCompiler.getType(valueTypeName, typeContext)

      private val signature = new Signature(
        ImmutableList.of(keyType, valueType),
        Type.UNIT
      )

      @Override
      def funcName = LocalAggregatorOperator.this.funcName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        config.description,
        ActionLevel.NO_ACTION,
        "aggregation key",
        "aggregation value"
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @Override
      @throws(classOf[SemanticCheckFailure])
      def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[LocalAggregatorContainer](funcName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[LocalAggregatorContainer],
            args: util.List[AnyRef]
          ): Future[AnyRef] = {
            val publisher = context.getRuntime.localAggregators(config)
            val key = args.get(0)
            val value = args.get(1)

            publisher.add(key, value)
            Future.Void
          }
        }
      }
    }
    List(post)
  }
}
