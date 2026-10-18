package com.twitter.botmaker.app.runtime

import com.twitter.botmaker.app.runtime.config.StdBotMakerAppConfigs
import com.twitter.botmaker.rule.BotMaker
import com.twitter.botmaker.rule.BotMakerRuntime
import com.twitter.botmaker.rule.EventProcessor
import com.twitter.botmaker.rule.config.SystemConfigs
import com.twitter.botmaker.runtime._

package config {
  import com.twitter.botmaker.runtime.config._

  class StdBotMakerAppConfigs
      extends SystemConfigs
      with LocalCounterConfigs
      with LocalStoreConfigs
      with ScheduledEventConfigs
      with BatchQueueConfigs
      with PackageConfigs
      with HttpEndpointConfigs
      with ThriftEndpointConfigs
      with EventBusConfigs
      with ScribePublisherConfigs
      with ManhattanConfigs
      with MemCachedConfigs
}

class StdBotMakerAppRuntime[CT <: StdBotMakerAppConfigs](
  botmaker: BotMaker[CT, _],
  registry: Seq[ServiceRegistry[Any]])
    extends BotMakerRuntime[CT](botmaker, registry)
    with LocalCounterContainer
    with LocalStoreContainer
    with EventProcessor
    with BatchQueueContainer
    with ScheduledEventContainer
    with EventBusContainer
    with ManhattanEndpointContainer
    with ThriftEndpointContainer
    with MemCachedEndpointContainer
