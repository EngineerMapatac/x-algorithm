package com.twitter.botmaker.runtime.config

import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava.EventPublisherConfig
import scala.reflect.ClassTag

object EventPublisher extends ConfigUnit[EventPublisherConfigs, EventPublisherConfig] {

  override def description: String = "Add an eventbus publisher."

  override def apply(
    context: Context[EventPublisherConfigs],
    config: EventPublisherConfig
  ): EventPublisherConfig = {
    context.getRuntime.addEventPublisher(config)
    config
  }
}

trait EventPublisherConfigs extends ServiceConfigs {

  import scala.collection.mutable.ArrayBuffer

  private final val eventPublishers: ArrayBuffer[EventPublisherConfig] =
    ArrayBuffer[EventPublisherConfig]()

  final def getEventPublishers: Seq[EventPublisherConfig] = eventPublishers.toSeq

  final def addEventPublisher(config: EventPublisherConfig): Unit = {

    unique(FuncName, "eb", config.funcName)

    validateFuncName(config.funcName)
    eventPublishers.append(config)
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("eventPublishers", eventPublishers) foreach { d =>
      eventPublishers foreach { eb => d.info(eb.funcName, eb.toString) }
    }
  }

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]
    eventPublishers foreach { eb =>
      val p = com.twitter.botmaker.runtime.EventPublisherOperator(eb)
      val u = p.toCompile
      builder.addCompilerUnit(u)
    }

    builder
  }
}
