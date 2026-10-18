package com.twitter.botmaker.runtime.config

import com.google.common.base.Strings
import com.google.common.util.concurrent.UncheckedExecutionException
import com.twitter.botmaker.compiler.ClassCache
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.runtime.ConfigFailure
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava.KafkaProducerConfig
import scala.reflect.ClassTag
import com.twitter.botmaker.runtime.TwitterRuntime.Thrift

object KafkaProducer extends ConfigUnit[KafkaProducerConfigs, KafkaProducerConfig] {

  override def description: String = "Add a kafka producer."

  override def apply(
    context: Context[KafkaProducerConfigs],
    config: KafkaProducerConfig
  ): KafkaProducerConfig = {
    context.getRuntime.addKafkaProducer(config)
    config
  }
}

trait KafkaProducerConfigs extends ServiceConfigs with EventTypeConfigs {

  private final var kafkaProducers = Seq[KafkaProducerConfig]()

  final def getKafkaProducers: Seq[KafkaProducerConfig] = kafkaProducers

  final def addKafkaProducer(config: KafkaProducerConfig): Unit = synchronized {

    unique(FuncName, "kf", config.funcName)

    validate(
      !Strings.isNullOrEmpty(config.topicName),
      s"topicName has to be configured in KafkaProducer ${config.funcName}"
    )

    validate(
      !Strings.isNullOrEmpty(config.dest),
      s"dest has to be configured in KafkaProducer ${config.funcName}"
    )

    try {
      val classObject = ClassCache.forName(config.valueThriftClass)
      validate(
        (classOf[Thrift[_, _]].isAssignableFrom(classObject)) || (classOf[String].isAssignableFrom(
          classObject)),
        s"The value class ${config.valueThriftClass}, is not supported"
      )
    } catch {
      case e: UncheckedExecutionException => {
        throw ConfigFailure(s"unknown value class ${config.valueThriftClass}")
      }
    }

    validateTypeName(config.keyType)

    kafkaProducers = kafkaProducers :+ config
  }

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)

    dbg.open("kafkaProducers", kafkaProducers) foreach { d =>
      kafkaProducers foreach { eb => d.info(eb.funcName, eb.toString) }
    }
  }

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]
    kafkaProducers foreach { kf =>
      val p = com.twitter.botmaker.runtime.KafkaProducerOperator(kf)
      val u = p.toCompile
      builder.addCompilerUnit(u)
    }

    builder
  }

}
