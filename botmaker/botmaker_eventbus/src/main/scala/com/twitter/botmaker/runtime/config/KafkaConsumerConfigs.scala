package com.twitter.botmaker.runtime.config

import com.google.common.base.Strings
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.thriftjava.BotMakerEventTypeConfig
import com.twitter.botmaker.runtime.thriftjava.KafkaConsumerConfig
import com.twitter.botmaker.runtime.thriftjava.ThriftFieldConfig
import com.twitter.botmaker.runtime.thriftjava.ThriftTypeConfig

object KafkaConsumer extends ConfigUnit[KafkaConsumerConfigs, KafkaConsumerConfig] {

  override def description: String = "Add a kafka consumer."

  override def apply(
    context: Context[KafkaConsumerConfigs],
    config: KafkaConsumerConfig
  ): KafkaConsumerConfig = {
    context.getRuntime.addKafkaConsumer(config)
    config
  }
}

trait KafkaConsumerConfigs extends ServiceConfigs with EventTypeConfigs {

  private final var kafkaConsumers = Seq[KafkaConsumerConfig]()

  final def getKafkaConsumers: Seq[KafkaConsumerConfig] = kafkaConsumers

  final def addKafkaConsumer(config: KafkaConsumerConfig): Unit = synchronized {

    unique(Endpoint, config.eventType)

    validateEventType(config.eventType)

    validate(
      config.seekStrategy match {
        case "BEGINNING" => true
        case "RESUME" => true
        case "REWIND" => true
        case "END" => true
        case _ => false
      },
      s"seekStrategy has to be BEGINNING, RESUME, REWIND or END in KafkaConsumer ${config.eventType}"
    )

    validate(
      !Strings.isNullOrEmpty(config.topicName),
      s"topicName has to be configured in KafkaConsumer ${config.eventType}"
    )

    validate(
      !Strings.isNullOrEmpty(config.dest),
      s"dest has to be configured in KafkaConsumer ${config.eventType}"
    )

    validate(
      !Strings.isNullOrEmpty(config.groupId),
      s"groupId has to be configured in KafkaConsumer ${config.eventType}"
    )

    validate(
      config.pollTimeoutMillis > 0,
      s"pollTimeoutMillis has to be greater than 0 in KafkaConsumer ${config.eventType}"
    )

    validate(
      !config.isSetRewindDurationMillis || config.rewindDurationMillis > 0,
      s"rewindDurationMillis has to be greater than 0 in KafkaConsumer ${config.eventType}"
    )

    validateTypeName(config.keyType)

    kafkaConsumers = kafkaConsumers :+ config

    val typeName = s"KafkaConsumer_${config.eventType}_ArgType"
    addThriftType(
      new ThriftTypeConfig()
        .setTypeName(typeName)
        .setClassName(config.valueThriftClass)
    )

    addEventType(
      new BotMakerEventTypeConfig()
        .setEventType(config.eventType)
        .setInputFeatures(
          ImmutableList.of(
            new ThriftFieldConfig()
              .setFieldName("topic")
              .setFieldId(1)
              .setFieldType("String"),
            new ThriftFieldConfig()
              .setFieldName("key")
              .setFieldId(2)
              .setFieldType("Object"),
            new ThriftFieldConfig()
              .setFieldName("value")
              .setFieldId(3)
              .setFieldType(typeName)
          )
        )
    )
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("kafkaConsumers", kafkaConsumers) foreach { d =>
      kafkaConsumers foreach { eb => d.info(eb.eventType, eb.toString) }
    }
  }
}
