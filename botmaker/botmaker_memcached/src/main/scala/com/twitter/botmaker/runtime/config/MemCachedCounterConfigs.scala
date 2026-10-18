package com.twitter.botmaker.runtime.config

import scala.collection.mutable.ArrayBuffer
import scala.reflect.ClassTag
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.runtime.ConfigFailure
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava.MemCachedCounterConfig

object MemCachedCounter extends ConfigUnit[MemCachedCounterConfigs, MemCachedCounterConfig] {

  override def description: String = "Add a Memcached counter."

  override def apply(
    context: Context[MemCachedCounterConfigs],
    config: MemCachedCounterConfig
  ): MemCachedCounterConfig = {
    context.getRuntime.addMemCachedCounter(config)
    config
  }
}

trait MemCachedCounterConfigs extends MemCachedEndpointConfigs {

  private final val memCachedCounters = ArrayBuffer[MemCachedCounterConfig]()

  def getMemeCachedCounters: Seq[MemCachedCounterConfig] = memCachedCounters.toSeq

  def addMemCachedCounter(config: MemCachedCounterConfig): Unit = {

    unique(FuncName, config.endpoint, config.funcName)

    validateFuncName(config.funcName)
    validateEndpoint(config.endpoint)
    validateDataset(config.dataset)

    memCachedCounters.append(config)
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("memCachedCounters", memCachedCounters) foreach { d =>
      memCachedCounters foreach { mc => d.info(mc.funcName, mc.toString) }
    }
  }

  override def validate: Seq[Throwable] = {
    val errors = memCachedCounters.toSeq collect {
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

    memCachedCounters foreach { mc =>
      val p = com.twitter.botmaker.runtime.MemCachedCounterOperator(mc)
      p.toCompile(tc).foreach(u => builder.addCompilerUnit(u))
    }
    builder
  }
}
