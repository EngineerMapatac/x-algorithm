package com.twitter.botmaker.runtime.config

import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.thriftjava.BotMakerEventTypeConfig
import com.twitter.botmaker.runtime.thriftjava.ThriftFieldConfig
import com.twitter.botmaker.runtime.thriftjava.ThriftTypeConfig
import com.twitter.botmaker.runtime.thriftjava.EventSubscriberConfig
import com.twitter.eventbus.client.EventBusSubscriber

object EventSubscriber extends ConfigUnit[EventSubscriberConfigs, EventSubscriberConfig] {

  override def description: String = "Add an eventbus subscriber."

  override def apply(
    context: Context[EventSubscriberConfigs],
    config: EventSubscriberConfig
  ): EventSubscriberConfig = {
    context.getRuntime.addEventSubscriber(config)
    config
  }
}

trait EventSubscriberConfigs extends ServiceConfigs with EventTypeConfigs {

  import scala.collection.mutable.ArrayBuffer

  private final val eventSubscribers = ArrayBuffer[EventSubscriberConfig]()

  final def getEventSubscribers: Seq[EventSubscriberConfig] = eventSubscribers.toSeq

  final def addEventSubscriber(config: EventSubscriberConfig): Option[BotMakerEventTypeConfig] = {

    unique(Endpoint, config.eventType)

    if (config.isSetSubscriberId) {
      unique(EventBusSubscriber, config.subscriberId)
    }

    if (config.isSetStreamName && config.isSetKafkaGroupId) {
      unique(EventBusSubscriber, config.streamName, config.kafkaGroupId)
    }

    validateEventType(config.eventType)

    validate(
      config.numThreads > 0,
      s"numTheads has to be greater than 0 in EventSubscriber ${config.eventType}"
    )

    validate(
      config.maxConcurrentEvents > 0,
      s"maxConcurrentEvents has to be greater than 0 in EventSubscriber ${config.eventType}"
    )

    validate(
      config.maxUnackedEvents > 0,
      s"maxUnackedEvents has to be greater than 0 in EventSubscriber ${config.eventType}"
    )

    validate(
      config.processTimeoutMillis > 0,
      s"processTimeoutMillis has to be greater than 0 in EventSubscriber ${config.eventType}"
    )

    validate(
      !(config.isSetStreamName && config.isSetSubscriberId)
        && (config.isSetSubscriberId || config.isSetStreamName),
      "either subscriberId or streamName (but not both) must be defined " +
        s"in EventSubscriber ${config.eventType}"
    )

    validate(
      !(config.isSetKafkaGroupId && config.isSetSubscriberId)
        && (config.isSetKafkaGroupId || config.isSetSubscriberId),
      "either kafkaGroupId or subscriberId (but not both) must be defined " +
        s"in EventSubscriber ${config.eventType}"
    )

    validate(
      config.isSetSubscriberId || (config.isSetStreamName && config.isSetKafkaGroupId),
      "Either subscriberId is defined or a combination of streamName and kafkaGroupId must " +
        s"be defined in EventSubscriber ${config.eventType}"
    )

    eventSubscribers.append(config)

    val typeName = s"EventSubscriber_${config.eventType}_ArgType"
    addThriftType(
      new ThriftTypeConfig()
        .setTypeName(typeName)
        .setClassName(config.thriftClass)
    )

    addEventType(
      new BotMakerEventTypeConfig()
        .setEventType(config.eventType)
        .setInputFeatures(
          ImmutableList.of(
            new ThriftFieldConfig()
              .setFieldName("event")
              .setFieldId(1)
              .setFieldType(typeName)
          )
        )
    )
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("eventSubscribers", eventSubscribers) foreach { d =>
      eventSubscribers foreach { eb => d.info(eb.eventType, eb.toString) }
    }
  }
}
