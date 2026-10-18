package com.twitter.botmaker.runtime.config

import scala.collection.mutable.ArrayBuffer
import scala.reflect.ClassTag
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.runtime.ConfigFailure
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava.MemCachedClientConfig

object MemCachedClient extends ConfigUnit[MemCachedClientConfigs, MemCachedClientConfig] {

  override def description: String = "Add a Memcached client."

  override def apply(
    context: Context[MemCachedClientConfigs],
    config: MemCachedClientConfig
  ): MemCachedClientConfig = {
    context.getRuntime.addMemCachedClient(config)
    config
  }
}

trait MemCachedClientConfigs extends MemCachedEndpointConfigs {

  private final val memCachedClients = ArrayBuffer[MemCachedClientConfig]()

  def getMemeCachedClients: Seq[MemCachedClientConfig] = memCachedClients.toSeq

  def addMemCachedClient(config: MemCachedClientConfig): Unit = {

    unique(FuncName, config.endpoint, config.funcName)

    validateFuncName(config.funcName)
    validateEndpoint(config.endpoint)
    validateDataset(config.dataset)

    memCachedClients.append(config)
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("memCachedClients", memCachedClients) foreach { d =>
      memCachedClients foreach { mc => d.info(mc.funcName, mc.toString) }
    }
  }

  override def validate: Seq[Throwable] = {
    val errors = memCachedClients.toSeq collect {
      case mc if !memCachedEndpoints.exists(_.endpoint == mc.endpoint) =>
        ConfigFailure(
          s"Endpoint ${mc.endpoint} in MemCachedClient ${mc.funcName} is not defined."
        )
    }

    errors ++ super.validate
  }

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {

    val builder = super.compilerBuilder[E]
    val tc = builder.typeContext()

    memCachedClients foreach { mc =>
      val p = com.twitter.botmaker.runtime.MemCachedClientOperator(mc)
      p.toCompile(tc).foreach(u => builder.addCompilerUnit(u))

    }
    builder
  }
}
