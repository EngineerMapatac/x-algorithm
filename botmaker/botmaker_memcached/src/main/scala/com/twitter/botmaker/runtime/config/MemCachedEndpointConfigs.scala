package com.twitter.botmaker.runtime.config

import scala.collection.mutable.ArrayBuffer
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.runtime.ConfigFailure
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.thriftjava.MemCachedEndpointConfig
import scala.collection.mutable

object MemCachedEndpoint extends ConfigUnit[MemCachedEndpointConfigs, MemCachedEndpointConfig] {

  override def description: String = "Add a Memcached endpoint."

  override def apply(
    context: Context[MemCachedEndpointConfigs],
    config: MemCachedEndpointConfig
  ): MemCachedEndpointConfig = {
    context.getRuntime.addMemCachedEndpoint(config)
    config
  }
}

trait MemCachedEndpointConfigs extends ServiceConfigs {

  final protected val memCachedEndpoints: ArrayBuffer[MemCachedEndpointConfig] =
    ArrayBuffer[MemCachedEndpointConfig]()

  final def getMemCachedEndpoints: Seq[MemCachedEndpointConfig] = memCachedEndpoints.toSeq

  final def addMemCachedEndpoint(config: MemCachedEndpointConfig): Unit = {
    unique(Endpoint, config.endpoint)

    validateEndpoint(config.endpoint)

    if (config.servicePath.isEmpty) {
      throw ConfigFailure(
        s"servicePath is not set for MemCachedEndpointConfig ${config.endpoint}"
      )
    }

    memCachedEndpoints.append(config)
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("memCachedEndpoints", memCachedEndpoints) foreach { d =>
      memCachedEndpoints foreach { mc => d.info(mc.endpoint, mc.toString) }
    }
  }
}
