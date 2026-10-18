package com.twitter.botmaker.runtime.config

import scala.collection.JavaConverters._
import scala.collection.mutable
import scala.reflect.ClassTag
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.runtime.thriftjava.BotMakerEventTypeConfig
import com.twitter.botmaker.runtime.thriftjava.ThriftFieldConfig
import com.twitter.botmaker.compiler.types.TypeCompiler
import com.twitter.botmaker.runtime.ConfigFailure
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.ServiceContainer

object EventType extends ConfigUnit[EventTypeConfigs, BotMakerEventTypeConfig] {

  override def description: String = "Create an event type config."

  override def apply(
    context: Context[EventTypeConfigs],
    config: BotMakerEventTypeConfig
  ): BotMakerEventTypeConfig = {
    context.getRuntime.addEventType(config)
    config
  }
}

object ThriftField extends ConfigUnit[ServiceConfigs, ThriftFieldConfig] {

  override def description: String = "Create a event thrift field config."

  override def apply(
    context: Context[ServiceConfigs],
    config: ThriftFieldConfig
  ): ThriftFieldConfig = {
    config
  }
}

trait EventTypeConfigs extends ServiceConfigs {

  final private val eventTypes = mutable.Map[String, BotMakerEventTypeConfig]()

  final def getEventTypes: Map[String, BotMakerEventTypeConfig] = eventTypes.toMap

  final def addEventType(config: BotMakerEventTypeConfig): Option[BotMakerEventTypeConfig] = {

    unique(EventType, config.eventType)
    validateEventType(config.eventType)

    config.inputFeatures.asScala foreach { fc => validateFieldName(fc.fieldName) }

    if (config.inputFeatures.asScala.map(_.fieldName).toSet.size != config.inputFeatures.size) {
      throw ConfigFailure(s"Field names in event type ${config.eventType} are not unique.")
    }

    if (config.inputFeatures.asScala.map(_.fieldId).toSet.size != config.inputFeatures.size) {
      throw ConfigFailure(s"Field Ids in event type ${config.eventType} are not unique.")
    }

    eventTypes.put(config.eventType, config)
  }

  override def debug(dbg: RuntimeDebugger): Unit = {

    super.debug(dbg)

    dbg.open("eventTypes", eventTypes) foreach { d =>
      eventTypes foreach {
        case (name, config) =>
          d.info(name, config.toString)
      }
    }
  }

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]
    val tc = builder.typeContext()

    eventTypes.values foreach { et =>
      et.inputFeatures.asScala foreach { fc =>
        val t = TypeCompiler.getType(fc.fieldType, tc)
        val n = s"${et.eventType}.${fc.fieldName}"
        builder.addInputFeature(n, t)
      }
    }

    builder
  }
}
