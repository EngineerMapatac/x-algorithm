package com.twitter.botmaker.runtime

import java.util.concurrent.atomic.AtomicBoolean
import scala.collection.JavaConverters._
import com.twitter.botmaker.rule.EventOutputCollector
import com.twitter.botmaker.rule.EventProcessor
import com.twitter.botmaker.runtime.config.KafkaConsumerConfigs
import com.twitter.botmaker.runtime.thriftjava.KafkaConsumerConfig
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.finatra.kafka.consumers.enableTlsAndKerberos
import com.twitter.finatra.kafka.consumers.FinagleKafkaConsumerBuilder
import com.twitter.finatra.kafka.consumers.FinagleKafkaConsumerConfig
import com.twitter.finatra.kafka.consumers.TwitterKafkaConsumerConfig
import com.twitter.finatra.kafka.domain.KafkaGroupId
import com.twitter.finatra.kafka.domain.KafkaTopic
import com.twitter.finatra.kafka.domain.SeekStrategy
import com.twitter.util.Duration
import com.twitter.util.Future
import com.twitter.util.Time

abstract class KafkaConsumerRegistry(val config: KafkaConsumerConfig)
    extends ServiceRegistry[KafkaConsumerConfig] {

  override val name: String = config.eventType
}

trait KafkaConsumerFactory extends TwitterRuntime with EventProcessor with KafkaSerdes {

  def build(config: KafkaConsumerConfig, scoped: StatsReceiver): KafkaConsumerRegistry = {
    val keyDeser = getKeyDeserializer(config.keyType)
    val valueDeser = getValueDeserializer(config.valueThriftClass)

    val seekStrage = config.seekStrategy match {
      case "BEGINNING" => SeekStrategy.BEGINNING
      case "RESUME" => SeekStrategy.RESUME
      case "REWIND" => SeekStrategy.REWIND
      case "END" => SeekStrategy.END
      case _ => throw ConfigFailure(s"invalid seekStrage ${config.seekStrategy}")
    }

    val kafkaConsumerConfig = com.twitter.finatra.kafka.consumers
      .KafkaConsumerConfig()
      .fromConfigMap(
        TwitterKafkaConsumerConfig().configMap ++
          config.configMap.asScala.toMap ++
          TwitterKafkaUtil.k8sTlsConfigOverrides(config.dest, enableTlsAndKerberos()))

    val finagleKafkaConsumerConfig = FinagleKafkaConsumerConfig[AnyRef, AnyRef](kafkaConsumerConfig)
    var builder = FinagleKafkaConsumerBuilder[AnyRef, AnyRef](finagleKafkaConsumerConfig)
      .dest(TwitterKafkaUtil.normalizeDest(config.dest, enableTlsAndKerberos()))
      .keyDeserializer(keyDeser)
      .valueDeserializer(valueDeser)
      .groupId(KafkaGroupId(config.groupId))
      .seekStrategy(seekStrage)
      .includeNodeMetrics(config.includeNodeMetrics)
      .clientId(config.clientId)
      .pollTimeout(Duration.fromMilliseconds(config.pollTimeoutMillis))
    if (config.isSetRewindDurationMillis) {
      builder = builder.rewindDuration(Duration.fromMilliseconds(config.rewindDurationMillis))
    }
    val consumer = builder.build()
    consumer.subscribe(Set(KafkaTopic(config.topicName)))

    val done = new AtomicBoolean(false)
    val attempts = scoped.counter("poll", "attempts")
    val successes = scoped.counter("poll", "successes")
    val failures = scoped.counter("poll", "failures")
    val commitAttempts = scoped.counter("commit", "attempts")
    val commitFailures = scoped.counter("commit", "failures")
    val commitSuccesses = scoped.counter("commit", "successes")

    def pollJob: Future[Unit] =
      Future.Unit
        .flatMap { _ =>
          attempts.incr()
          consumer
            .poll().onSuccess { _ => successes.incr() }.onFailure { t =>
              logger.warning(this, "failed to poll", t)
              failures.incr()
            }
        }.flatMap { result =>
          Future
            .collectToTry {
              result.records(config.topicName).iterator.asScala.toSeq.map { cr =>
                checkEvent(
                  eventType = config.eventType,
                  inputFeatures = {
                    case "topic" => config.topicName
                    case "key" => cr.key
                    case "value" => cr.value
                  },
                  new EventOutputCollector
                ).liftToTry
              }
            }.flatMap { _ =>
              commitAttempts.incr()
              consumer
                .commit()
                .onFailure { t =>
                  logger.warning(this, "failed to commit", t)
                  commitFailures.incr()
                }.onSuccess { _ => commitSuccesses.incr() }.liftToTry
            }
        }.liftToTry.flatMap { _ =>
          if (done.get) {
            Future.Done
          } else {
            pollJob
          }
        }

    val task = pollJob

    new KafkaConsumerRegistry(config) {

      override def close(deadline: Time): Future[Unit] = {
        done.set(true)
        task.flatMap { _ => consumer.close(deadline) }.unit
      }
    }
  }
}

trait KafkaConsumerContainer extends ServiceContainer with KafkaConsumerFactory {
  container =>

  override type _CONFIGS_TYPE <: KafkaConsumerConfigs

  private def isReusable(config: KafkaConsumerConfig) =
    isReusable[KafkaConsumerConfig, KafkaConsumerRegistry](config.eventType, config)

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val kafkaConsumers = configs.getKafkaConsumers map {
      case config if isReusable(config) =>
        logger.info(s"reuse KafkaConsumer ${config.eventType}")
        serviceRegistry[KafkaConsumerRegistry](config.eventType).get
      case config =>
        logger.info(s"create KafkaConsumer ${config.eventType}")
        build(config, scoped.scope("KafkaConsumer"))
    }
    kafkaConsumers ++ super.build(configs, scoped)
  }
}
