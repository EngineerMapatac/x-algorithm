package com.twitter.botmaker.runtime.config

import scala.collection.mutable.ArrayBuffer
import scala.reflect.ClassTag
import com.twitter.botmaker.Context
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.runtime.ConfigFailure
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava.ScribePublisherConfig
import com.twitter.util.Try

object ScribePublisher extends ConfigUnit[ScribePublisherConfigs, ScribePublisherConfig] {

  override def description: String = "Add a scribe publisher."

  override def apply(
    context: Context[ScribePublisherConfigs],
    config: ScribePublisherConfig
  ): ScribePublisherConfig = {
    context.getRuntime.addScribePublisher(config)
    config
  }
}

trait ScribePublisherConfigs extends ThriftEndpointConfigs {

  private final val scribePublishers = ArrayBuffer[ScribePublisherConfig]()

  final def getScribePublishers: Seq[ScribePublisherConfig] = scribePublishers.toSeq

  final def addScribePublisher(config: ScribePublisherConfig): Unit = {

    unique(FuncName, "sp", config.funcName)

    validateFuncName(config.funcName)
    validateEndpoint(config.thriftEndpoint)

    scribePublishers.append(config)
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("scribePublishers", scribePublishers) foreach { d =>
      scribePublishers foreach { sc => d.info(sc.funcName, sc.toString) }
    }
  }

  override def validate: Seq[Throwable] = {
    val errors = scribePublishers.toSeq collect {
      case sc if getThriftEndpoint(sc.thriftEndpoint).isEmpty =>
        ConfigFailure(
          s"Thrift endpoint ${sc.thriftEndpoint} for ScribePublisher ${sc.funcName} not defined"
        )
    }

    val errors2 = scribePublishers.toSeq map { sc =>
      Try {
        validateClassName(sc.thriftClass)
      }
    } filter {
      _.isThrow
    } map {
      _.throwable
    }

    errors ++ errors2 ++ super.validate
  }

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]

    scribePublishers foreach { sc =>
      val p = com.twitter.botmaker.runtime.ScribePublisherOperator(sc)
      val u = p.toCompile
      builder.addCompilerUnit(u)
    }

    builder
  }
}
