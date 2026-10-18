package com.twitter.botmaker.runtime.config

import scala.collection.mutable
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.runtime.ConfigFailure
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.thriftjava.BotMakerEventTypeConfig
import com.twitter.botmaker.runtime.thriftjava.ScheduledEventConfig

object ScheduledEvent extends ConfigUnit[ScheduledEventConfigs, ScheduledEventConfig] {

  override def description: String = "Create a schedule event."

  override def apply(
    context: Context[ScheduledEventConfigs],
    config: ScheduledEventConfig
  ): ScheduledEventConfig = {
    context.getRuntime.addScheduledEvent(config)
    config
  }
}

trait ScheduledEventConfigs extends ServiceConfigs with EventTypeConfigs {

  private final val scheduledEvents = mutable.ArrayBuffer[ScheduledEventConfig]()

  final def getScheduledEvents: Seq[ScheduledEventConfig] = scheduledEvents.toSeq

  final def addScheduledEvent(config: ScheduledEventConfig): Option[BotMakerEventTypeConfig] = {

    unique(ScheduledEvent, config.eventType)
    unique(Endpoint, config.eventType)

    if (config.intervalMillis <= 0) {
      throw ConfigFailure(
        s"ScheduledEvent ${config.eventType} intervalMillis must be a postivie value."
      )
    }

    scheduledEvents.append(config)

    addEventType(
      new BotMakerEventTypeConfig()
        .setEventType(config.eventType)
    )

  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("scheduledEvents", scheduledEvents) foreach { d =>
      scheduledEvents foreach { se => d.info(se.eventType, se.toString) }
    }
  }
}
