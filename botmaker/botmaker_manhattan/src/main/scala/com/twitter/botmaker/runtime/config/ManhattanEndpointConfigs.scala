package com.twitter.botmaker.runtime.config

import scala.collection.mutable.ArrayBuffer

import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.thriftjava.ManhattanEndpointConfig

object ManhattanEndpoint extends ConfigUnit[ManhattanEndpointConfigs, ManhattanEndpointConfig] {

  override def description: String = "Add a Manhattan endpoint."

  override def apply(
    context: Context[ManhattanEndpointConfigs],
    config: ManhattanEndpointConfig
  ): ManhattanEndpointConfig = {
    context.getRuntime.addManhattanEndpoint(config)
    config
  }
}

trait ManhattanEndpointConfigs extends ServiceConfigs {

  private final val manhattanEndpoints = ArrayBuffer[ManhattanEndpointConfig]()

  final def getManhattanEndpoints: Seq[ManhattanEndpointConfig] = manhattanEndpoints

  final def getManhattanEndpoint(endpoint: String): Option[ManhattanEndpointConfig] = {
    manhattanEndpoints.find(_.endpoint == endpoint)
  }

  final def addManhattanEndpoint(config: ManhattanEndpointConfig): Unit = {

    unique(Endpoint, config.endpoint)

    validateEndpoint(config.endpoint)

    manhattanEndpoints.append(config)
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("manhattanEndpoints", manhattanEndpoints) foreach { d =>
      manhattanEndpoints foreach { me => d.info(me.endpoint, me.toString) }
    }
  }
}
