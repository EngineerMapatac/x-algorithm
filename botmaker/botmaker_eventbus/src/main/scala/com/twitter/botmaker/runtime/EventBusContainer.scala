package com.twitter.botmaker.runtime

import com.twitter.botmaker.runtime.config.EventBusConfigs

trait EventBusContainer
    extends EventPublisherContainer
    with EventSubscriberContainer
    with KafkaProducerContainer
    with KafkaConsumerContainer {
  override type _CONFIGS_TYPE <: EventBusConfigs

}
