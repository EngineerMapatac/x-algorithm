package com.twitter.botmaker.rule

import com.twitter.util.Future

trait EventProcessor {

  def processEvent(
    eventType: String,
    inputFeatures: InputProvider = InputProvider.empty,
    outputCollector: OutputCollector = OutputCollector.empty
  ): Future[Unit]

  def checkEvent[T <: EventOutputCollector](
    eventType: String,
    inputFeatures: InputProvider = InputProvider.empty,
    outputCollector: T
  ): Future[T]
}
