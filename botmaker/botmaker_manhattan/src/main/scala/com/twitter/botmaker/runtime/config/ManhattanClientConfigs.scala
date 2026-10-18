package com.twitter.botmaker.runtime.config

import scala.collection.mutable.ArrayBuffer
import scala.reflect.ClassTag
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.runtime.ConfigFailure
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava.ManhattanClientConfig

object ManhattanClient extends ConfigUnit[ManhattanClientConfigs, ManhattanClientConfig] {

  override def description: String = "Add a Manhattan client."

  override def apply(
    context: Context[ManhattanClientConfigs],
    config: ManhattanClientConfig
  ): ManhattanClientConfig = {
    context.getRuntime.addManhattanClient(config)
    config
  }
}

trait ManhattanClientConfigs extends ManhattanEndpointConfigs {

  private final val manhattanClients = ArrayBuffer[ManhattanClientConfig]()

  final def getManhattanClients: Seq[ManhattanClientConfig] = manhattanClients

  final def addManhattanClient(config: ManhattanClientConfig): Unit = {

    unique(FuncName, config.endpoint, config.funcName)

    validateEndpoint(config.endpoint)
    validateFuncName(config.funcName)
    validateDataset(config.dataset)

    if (config.isSetTtlMillis) {
      validate(config.ttlMillis > 0, "ttlMillis has to be greater than 0")
    }

    manhattanClients.append(config)
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("manhattanClients", manhattanClients) foreach { d =>
      manhattanClients foreach { mc => d.info(mc.funcName, mc.toString) }
    }
  }

  override def validate: Seq[Throwable] = {
    val errors = manhattanClients.toSeq collect {
      case mc if getManhattanEndpoint(mc.endpoint).isEmpty =>
        ConfigFailure(
          s"endpoint ${mc.endpoint} is not defined for Manhattan Client ${mc.funcName}"
        )
    }
    errors ++ super.validate
  }

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]
    val tc = builder.typeContext()

    manhattanClients foreach { mc =>
      val p = com.twitter.botmaker.runtime.ManhattanClientOperator(mc)
      p.toCompile(tc).foreach(u => builder.addCompilerUnit(u))
    }

    builder
  }
}
