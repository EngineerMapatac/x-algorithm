package com.twitter.botmaker.runtime.config

import java.util
import scala.collection.JavaConverters._
import scala.reflect.ClassTag
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.tuples.Tuple
import com.twitter.botmaker.runtime.LocalAggregatorActor
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava._
import com.twitter.util.Future

trait AggregatorProcessor {
  def process(stats: util.List[Tuple]): Future[Unit]
}

abstract class AggregatorUnit[I, M, O]
    extends ConfigUnit[LocalAggregatorConfigs, LocalAggregatorConfig] {

  def keyType(config: LocalAggregatorConfig): String = {
    if (config.keyTypes.size == 1) {
      config.keyTypes.get(0)
    } else {
      config.keyTypes.asScala.mkString("Tuple<", ",", ">")
    }
  }

  def valueType(config: LocalAggregatorConfig): String

  def resultType(config: LocalAggregatorConfig): String

  def zero(config: LocalAggregatorConfig): M
  def plus(config: LocalAggregatorConfig, stat: M, amount: I): Unit
  def flush(config: LocalAggregatorConfig, stat: M): O

  def filter(config: LocalAggregatorConfig, stat: M) = true

  override def apply(
    context: Context[LocalAggregatorConfigs],
    config: LocalAggregatorConfig
  ): LocalAggregatorConfig = {

    val kt = keyType(config)
    val vt = valueType(config)
    val rt = resultType(config)

    val actor = LocalAggregatorActor(this, config, kt, vt, rt)
    context.getRuntime.addLocalAggregator(actor)
    config
  }
}

trait LocalAggregatorConfigs extends ServiceConfigs with EventTypeConfigs {

  import scala.collection.mutable.ArrayBuffer

  private final val localAggregators = ArrayBuffer[LocalAggregatorActor]()

  def getLocalAggregators: Seq[LocalAggregatorActor] = localAggregators.toSeq

  def addLocalAggregator(actor: LocalAggregatorActor): Option[BotMakerEventTypeConfig] = {

    val config = actor.config

    unique(Endpoint, config.eventType)
    unique(BatchQueue, config.eventType)

    validateEventType(config.eventType)

    validate(
      config.flushIntervalMillis > 0,
      s"flushIntervalMillis must be greater than 0 in LocalAggregator ${config.eventType}"
    )
    validate(
      config.flushBatchSize > 0,
      s"flushBatchSize must be greater than 0 in LocalAggregator ${config.eventType}"
    )
    validate(
      config.flushThreshold >= 0,
      s"flushThreshold must be non negative in LocalAggregator ${config.eventType}"
    )
    validate(
      config.flushSlices > 0,
      s"flushSlices must be greater than 0 in LocalAggregator ${config.eventType}"
    )
    validate(
      config.maximumCacheItems > 0,
      s"maximumCacheItems must be greater than 0 in LocalAggregator ${config.eventType}"
    )
    validate(
      config.cacheExpirationMillis > 0,
      s"cacheExpirationMillis must be greater than 0 in LocalAggregator ${config.eventType}"
    )

    localAggregators.append(actor)

    addEventType(actor.eventType)
  }

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]
    val tc = builder.typeContext()

    localAggregators foreach {
      case agg => agg.compilerUnits(tc) foreach { builder.addCompilerUnit(_) }
    }

    builder
  }
}
