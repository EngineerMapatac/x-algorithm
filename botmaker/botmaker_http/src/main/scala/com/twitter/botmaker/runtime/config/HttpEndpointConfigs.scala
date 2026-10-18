package com.twitter.botmaker.runtime.config

import scala.collection.mutable.ArrayBuffer
import scala.reflect.ClassTag
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.compiler.CompilerUnit
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava.HttpEndpointConfig

object HttpEndpoint extends ConfigUnit[HttpEndpointConfigs, HttpEndpointConfig] {

  override def description: String = "Create an Http endpoint."

  override def apply(
    context: Context[HttpEndpointConfigs],
    config: HttpEndpointConfig
  ): HttpEndpointConfig = {
    context.getRuntime.addHttpEndpoint(config)
    config
  }
}

trait HttpEndpointConfigs extends ServiceConfigs {

  private final val httpEndpoints: ArrayBuffer[HttpEndpointConfig] =
    ArrayBuffer[HttpEndpointConfig]()

  final def getHttpEndpoints: Seq[HttpEndpointConfig] = httpEndpoints.toSeq

  final def addHttpEndpoint(config: HttpEndpointConfig): Unit = {

    unique(Endpoint, config.endpoint)

    validateEndpoint(config.endpoint)

    httpEndpoints.append(config)
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("httpEndpoints", httpEndpoints) foreach { d =>
      httpEndpoints foreach { he => d.info(he.endpoint, he.toString) }
    }
  }

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]

    httpEndpoints foreach { endpoint: HttpEndpointConfig =>
      val compilerUnit: CompilerUnit =
        com.twitter.botmaker.runtime.HttpClientOperator(endpoint).toCompile
      builder.addCompilerUnit(compilerUnit)
    }

    builder
  }
}
