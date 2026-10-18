package com.twitter.botmaker.runtime.config

trait EventBusConfigs
    extends EventPublisherConfigs
    with EventSubscriberConfigs
    with KafkaProducerConfigs
    with KafkaConsumerConfigs
