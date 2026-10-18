package com.twitter.botmaker.runtime.config

import scala.collection.mutable
import scala.reflect.ClassTag
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava.LocalStoreConfig

object LocalStore extends ConfigUnit[LocalStoreConfigs, LocalStoreConfig] {

  override def description: String = "Create a local store."

  override def apply(
    context: Context[LocalStoreConfigs],
    config: LocalStoreConfig
  ): LocalStoreConfig = {
    context.getRuntime.addLocalStore(config)
    config
  }
}

trait LocalStoreConfigs extends ServiceConfigs {

  private final val localStores = mutable.ArrayBuffer[LocalStoreConfig]()

  final def getLocalStores: Seq[LocalStoreConfig] = localStores.toSeq

  final def addLocalStore(config: LocalStoreConfig): Unit = {

    unique(FuncName, "local", config.funcName)
    unique(LocalStore, config.funcName)

    validateFuncName(config.funcName)

    if (config.isSetCacheExpirationMillis) {
      validate(config.cacheExpirationMillis > 0, "cacheExpirationMillis must be greater than 0")
    }

    if (config.isSetMaximumCacheItems) {
      validate(config.maximumCacheItems > 0, "maximumCacheItems must be greater than 0")
    }

    localStores.append(config)
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("localStores", localStores) foreach { d =>
      localStores foreach { mc => d.info(mc.funcName, mc.toString) }
    }
  }

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]
    val tc = builder.typeContext()

    localStores foreach {
      case mc if mc.keyTypes.size > 0 =>
        val p = com.twitter.botmaker.runtime.LocalStoreOperator(mc)
        p.toCompile(tc).foreach(u => builder.addCompilerUnit(u))
      case mc if mc.keyTypes.size == 0 =>
        val p = com.twitter.botmaker.runtime.LocalVarOperator(mc)
        p.toCompile(tc).foreach(u => builder.addCompilerUnit(u))
    }

    builder
  }
}
