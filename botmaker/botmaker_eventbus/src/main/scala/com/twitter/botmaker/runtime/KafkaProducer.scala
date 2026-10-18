package com.twitter.botmaker.runtime

import java.lang.{Long => JLong}
import java.util.{List => JList}
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.ThreadPoolExecutor
import java.util.concurrent.TimeUnit
import scala.collection.JavaConverters._
import com.google.common.collect.ImmutableList
import org.apache.kafka.clients.producer.ProducerConfig
import org.apache.kafka.clients.producer.ProducerRecord
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.GeneralNode
import com.twitter.botmaker.runtime.TwitterRuntime.Thrift
import com.twitter.botmaker.runtime.config.KafkaProducerConfigs
import com.twitter.botmaker.runtime.thriftjava.KafkaProducerConfig
import com.twitter.concurrent.NamedPoolThreadFactory
import com.twitter.eventbus.client.mock.mockEventBus
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.finatra.kafka.domain.AckMode
import com.twitter.finatra.kafka.interceptors.PublishTimeProducerInterceptor
import com.twitter.finatra.kafka.producers.FinagleKafkaProducerBuilder
import com.twitter.finatra.kafka.producers.FinagleKafkaProducerConfig
import com.twitter.finatra.kafka.producers.KafkaProducerBase
import com.twitter.finatra.kafka.producers.NullKafkaProducer
import com.twitter.finatra.kafka.producers.TwitterKafkaProducerConfig
import com.twitter.finatra.kafka.producers.enableTlsAndKerberos
import com.twitter.finatra.kafka.producers.{KafkaProducerConfig => TFinagleKafkaProducerConfig}
import com.twitter.finatra.kafka.stats.KafkaFinagleMetricsReporter
import com.twitter.finagle.util.DefaultTimer
import com.twitter.util.Duration
import com.twitter.util.Future
import com.twitter.util.FuturePool
import com.twitter.util.FuturePools
import com.twitter.util.Time
import com.twitter.util.Timer
import org.apache.kafka.common.metrics.Sensor.RecordingLevel

import scala.runtime.BoxedUnit

case class KafkaProducerRegistry(
  config: KafkaProducerConfig,
  producer: KafkaProducerBase[AnyRef, AnyRef],
  futurePool: FuturePool)
    extends ServiceRegistry[KafkaProducerConfig] {

  override val name: String = config.funcName

  override def close(deadline: Time): Future[Unit] = producer.close(deadline)
}

trait KafkaProducerFactory extends TwitterRuntime with KafkaSerdes {

  def build(
    config: KafkaProducerConfig,
    scoped: StatsReceiver
  ): KafkaProducerBase[AnyRef, AnyRef] = {
    val keySer = getKeySerializer(config.keyType)
    val valueSer = getValueSerializer(config.valueThriftClass)

    val defaultConfigValues = Map(
      ProducerConfig.DELIVERY_TIMEOUT_MS_CONFIG -> "20000",
      ProducerConfig.ACKS_CONFIG -> AckMode.ALL.toString,
      ProducerConfig.MAX_BLOCK_MS_CONFIG -> "3000",
      ProducerConfig.REQUEST_TIMEOUT_MS_CONFIG -> "10000"
    )

    val configMap = TwitterKafkaProducerConfig().configMap ++
      defaultConfigValues ++
      config.configMap.asScala.toMap ++
      TwitterKafkaUtil.k8sTlsConfigOverrides(config.dest, enableTlsAndKerberos())

    val kafkaProducerConfig =
      TFinagleKafkaProducerConfig()
        .fromConfigMap(configMap)
        .metricReporter[KafkaFinagleMetricsReporter]
        .metricsRecordingLevel(RecordingLevel.INFO)
        .metricsSampleWindow(Duration.fromSeconds(60))
        .interceptor[PublishTimeProducerInterceptor]

    val finageKafkaProducerConfig = FinagleKafkaProducerConfig[AnyRef, AnyRef](kafkaProducerConfig)
    if (mockEventBus()) {
      new NullKafkaProducer[AnyRef, AnyRef]
    } else {
      FinagleKafkaProducerBuilder[AnyRef, AnyRef](finageKafkaProducerConfig)
        .dest(TwitterKafkaUtil.normalizeDest(config.dest, enableTlsAndKerberos()))
        .keySerializer(keySer)
        .valueSerializer(valueSer)
        .clientId(config.clientId)
        .includeNodeMetrics(config.includeNodeMetrics)
        .statsReceiver(scoped)
        .build()
    }
  }
}

trait KafkaFuturePoolFactory extends TwitterRuntime {
  def buildFuturePool(config: KafkaProducerConfig): FuturePool = {
    Option(config.futurePoolConfig) match {
      case Some(poolConfig) =>
        logger.info(s"Built future pool for ${config.funcName} ${config.futurePoolConfig}")
        FuturePools.newFuturePool(
          new ThreadPoolExecutor(
            poolConfig.corePoolSize,
            poolConfig.maxPoolSize,
            poolConfig.keepAliveTimeMillis,
            TimeUnit.MILLISECONDS,
            new LinkedBlockingQueue[Runnable](poolConfig.queueCapacity),
            new NamedPoolThreadFactory(config.funcName, false)))
      case None =>
        logger.info(s"Using default future pool for ${config.funcName}")
        FuturePool.interruptibleUnboundedPool
    }
  }
}

trait KafkaProducerContainer
    extends ServiceContainer
    with KafkaProducerFactory
    with KafkaFuturePoolFactory {
  container =>

  override type _CONFIGS_TYPE <: KafkaProducerConfigs

  private def isReusable(config: KafkaProducerConfig) =
    isReusable[KafkaProducerConfig, KafkaProducerRegistry](config.funcName, config)

  def kafkaProducers(config: KafkaProducerConfig): KafkaProducerBase[AnyRef, AnyRef] = {
    serviceRegistry[KafkaProducerRegistry](config.funcName).get.producer
  }

  def futurePool(config: KafkaProducerConfig): FuturePool = {
    serviceRegistry[KafkaProducerRegistry](config.funcName).get.futurePool
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val kafkaProducers = configs.getKafkaProducers map {
      case config if isReusable(config) =>
        logger.info(s"reuse KafkaProducer ${config.funcName}")
        serviceRegistry[KafkaProducerRegistry](config.funcName).get
      case config =>
        logger.info(s"create KafkaProducerRegistry ${config.funcName}")

        val producer = build(config, scoped.scope("KafkaProducer"))

        val futurePool = buildFuturePool(config)

        KafkaProducerRegistry(config, producer, futurePool)
    }
    kafkaProducers ++ super.build(configs, scoped)
  }
}

case class KafkaProducerOperator(config: KafkaProducerConfig) {

  private val ns = "kf."
  private val funcName: String = ns + config.funcName
  implicit val timer: com.twitter.util.Timer = DefaultTimer.getInstance

  def toCompile: CompilerUnit = new CompilerUnit {

    private def getValueClassObjectType(valueClass: String): Type = {
      val classObject = ClassCache.forName(valueClass)
      if (classOf[Thrift[_, _]].isAssignableFrom(classObject)) {
        Type.thriftOf(classObject.asInstanceOf[Class[Thrift[_, _]]])
      } else if (classOf[String].isAssignableFrom(classObject)) {
        Type.STRING
      } else {
        throw ConfigFailure(s"The value class ${config.valueThriftClass}, is not supported")
      }
    }

    private val signature = new Signature(
      ImmutableList.of[Type](Type.OBJECT, getValueClassObjectType(config.valueThriftClass)),
      ImmutableList.of[Type](Type.LONG),
      Type.UNIT
    )

    override def funcName: String = KafkaProducerOperator.this.funcName

    override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
      funcName,
      signature,
      s"publish an event as ${config.valueThriftClass} object to kafka cluster: ${config.description}",
      ActionLevel.PROD_OTHER_ACTION,
      "key",
      "value",
      "partition index"
    )

    override def fingerprint(): Fingerprint = Fingerprint.of(config)

    @throws(classOf[SemanticCheckFailure])
    override def createASTNode(
      children: ImmutableList[ASTNode[_]]
    ): GeneralNode[KafkaProducerContainer] = {

      new GeneralNode[KafkaProducerContainer](funcName, children) {

        override def getSignature: Signature = signature

        override protected def getCacheLevel = CacheLevel.Never

        override protected def computeClassName: String = funcName

        override protected def evaluate(
          context: Context[KafkaProducerContainer],
          args: JList[AnyRef]
        ): Future[AnyRef] = {
          val producer = context.getRuntime.kafkaProducers(config)
          val futurePool = context.getRuntime.futurePool(config)
          val key = args.get(0)
          val value = args.get(1)
          val partitionIdx: Option[Integer] = if (args.size > 2) {
            Some(args.get(2).asInstanceOf[JLong].intValue)
          } else {
            None
          }

          val producerRecord = new ProducerRecord[AnyRef, AnyRef](
            config.topicName,
            partitionIdx.orNull,
            context.nowMillis,
            key,
            value
          )
          futurePool {
            val baseFuture = producer.send(producerRecord).map(_ => BoxedUnit.UNIT)

            Option(config.futurePoolConfig) match {
              case Some(poolConfig) =>
                baseFuture.within(Duration.fromMilliseconds(poolConfig.timeoutMs))
              case None =>
                baseFuture
            }
          }.flatten
        }
      }
    }
  }
}
