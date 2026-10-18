package com.twitter.botmaker.runtime

import com.twitter.botmaker.compiler._
import com.twitter.botmaker.rule.EventOutputCollector
import com.twitter.botmaker.rule.EventProcessor
import com.twitter.botmaker.runtime.EventSubscriberRegistry.configName
import com.twitter.botmaker.runtime.TwitterRuntime.Thrift
import com.twitter.botmaker.runtime.config.EventSubscriberConfigs
import com.twitter.botmaker.runtime.thriftjava.EventSubscriberConfig
import com.twitter.eventbus.client.EventBusSubscriber
import com.twitter.eventbus.client.EventBusSubscriberBuilder
import com.twitter.eventbus.client.PartitionedEventId
import com.twitter.eventbus.readproxy.thriftscala.EventId
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util.Duration
import com.twitter.util.Future
import com.twitter.util.Time
import org.apache.thrift.TBase

object EventSubscriberRegistry {

  def configName(config: EventSubscriberConfig): String = {
    if (config.isSetSubscriberId) {
      config.subscriberId
    } else {
      config.streamName + "_" + config.kafkaGroupId
    }
  }
}

case class EventSubscriberRegistry(
  config: EventSubscriberConfig,
  subscriber: EventBusSubscriber[Thrift[_, _]])
    extends ServiceRegistry[EventSubscriberConfig] {

  override val name: String = configName(config)

  override def close(deadline: Time): Future[Unit] = subscriber.close(deadline)
}

trait EventBusSubscriberFactory extends TwitterRuntime with EventProcessor {

  def build(
    config: EventSubscriberConfig,
    scoped: StatsReceiver
  ): EventBusSubscriber[TBase[_, _]] = {
    val cls = ClassCache.forName(config.thriftClass).asInstanceOf[Class[Thrift[_, _]]]

    var builder = EventBusSubscriberBuilder()
      .numThreads(config.numThreads)
      .maxConcurrentEvents(config.maxConcurrentEvents)
      .maxUnackedEvents(config.maxUnackedEvents)
      .processTimeout(Duration.fromMilliseconds(config.processTimeoutMillis))
      .skipToLatest(config.skipToLatest)
      .statsReceiver(scoped)
      .serviceIdentifier(serviceIdentifier)
      .thriftStruct(cls)

    if (config.isSetSubscriberId) {
      builder = builder.subscriberId(config.subscriberId)
    }

    if (config.isSetFromAllZones) {
      builder = builder.fromAllZones(config.fromAllZones)
    }

    if (config.isSetDelayMillis) {
      builder = builder.delay(Duration.fromMilliseconds(config.delayMillis))
    }

    if (config.isSetGroupId) {
      builder = builder.groupId(config.groupId)
    }

    if (config.isSetKafkaDest) {
      builder = builder.kafkaDest(config.kafkaDest)
    }

    if (config.isSetStreamName) {
      builder = builder.streamName(config.streamName)
    }

    if (config.isSetKafkaGroupId) {
      builder = builder.kafkaGroupId(config.kafkaGroupId)
    }

    builder.buildWithEnvelope(event =>
      checkEvent(
        eventType = config.eventType,
        inputFeatures = {
          case "event" => event.event
          case "partition" if event.id.isInstanceOf[PartitionedEventId] =>
            event.id.asInstanceOf[PartitionedEventId].partitionId
          case "seq"
              if event.id.isInstanceOf[PartitionedEventId] &&
                event.id.asInstanceOf[PartitionedEventId].eventId.isInstanceOf[EventId] =>
            val partitionedEventId = event.id.asInstanceOf[PartitionedEventId]
            Long.box(partitionedEventId.eventId.asInstanceOf[EventId].sequenceId)
        },
        new EventOutputCollector
      ).unit)
  }
}

trait EventSubscriberContainer extends ServiceContainer with EventBusSubscriberFactory {
  container =>

  override type _CONFIGS_TYPE <: EventSubscriberConfigs

  private def isReusable(config: EventSubscriberConfig) =
    isReusable[EventSubscriberConfig, EventSubscriberRegistry](configName(config), config)

  def eventSubscribers(config: EventSubscriberConfig): EventBusSubscriber[Thrift[_, _]] = {
    serviceRegistry[EventSubscriberRegistry](configName(config)).get.subscriber
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val eventSubscribers = configs.getEventSubscribers map {
      case config if isReusable(config) =>
        logger.info(s"reuse EventBusSubscriber ${configName(config)}")
        serviceRegistry[EventSubscriberRegistry](configName(config)).get
      case config =>
        logger.info(s"create EventBusSubscriber ${configName(config)}")
        val subscriber = build(config, scoped.scope("EventSubscriber"))
        EventSubscriberRegistry(config, subscriber)
    }
    eventSubscribers ++ super.build(configs, scoped)
  }
}
