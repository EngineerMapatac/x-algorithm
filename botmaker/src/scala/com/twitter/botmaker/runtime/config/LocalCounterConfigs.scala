package com.twitter.botmaker.runtime.config

import scala.collection.mutable
import scala.reflect.ClassTag
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava.LocalCounterConfig

object LocalCounter extends ConfigUnit[LocalCounterConfigs, LocalCounterConfig] {

  override def description: String = "Create a local counter."

  override def apply(
    context: Context[LocalCounterConfigs],
    config: LocalCounterConfig
  ): LocalCounterConfig = {
    context.getRuntime.addLocalCounter(config)
    config
  }
}

trait LocalCounterConfigs extends ServiceConfigs {

  private final val localCounters = mutable.ArrayBuffer[LocalCounterConfig]()

  final def getLocalCounters: Seq[LocalCounterConfig] = localCounters.toSeq

  final def addLocalCounter(config: LocalCounterConfig): Unit = {

    unique(FuncName, "local", config.funcName)
    unique(LocalCounter, config.funcName)

    validateFuncName(config.funcName)

    if (config.isSetCacheExpirationMillis) {
      validate(config.cacheExpirationMillis > 0, "cacheExpirationMillis must be greater than 0")
    }

    if (config.isSetMaximumCacheItems) {
      validate(config.maximumCacheItems > 0, "maximumCacheItems must be greater than 0")
    }

    localCounters.append(config)
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("localCounters", localCounters) foreach { d =>
      localCounters foreach { mc => d.info(mc.funcName, mc.toString) }
    }
  }

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]
    val tc = builder.typeContext()

    localCounters foreach {
      case mc if mc.keyTypes.size > 0 =>
        val p = com.twitter.botmaker.runtime.LocalCounterStoreOperator(mc)
        p.toCompile(tc).foreach(u => builder.addCompilerUnit(u))
      case mc if mc.keyTypes.size == 0 =>
        val p = com.twitter.botmaker.runtime.LocalCounterOperator(mc)
        p.toCompile.foreach(u => builder.addCompilerUnit(u))
    }

    builder
  }
}
