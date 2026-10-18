package com.twitter.botmaker.runtime.config

import scala.collection.mutable
import scala.reflect.ClassTag
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava._
import java.text.SimpleDateFormat

object LogWriter extends ConfigUnit[LogWriterConfigs, LogWriterConfig] {

  val timeFormatter: SimpleDateFormat = com.twitter.util.TwitterDateFormat("yyyy-MM-dd HH:mm:ss")

  override def description: String = "Define a local file writer."

  override def apply(
    context: Context[LogWriterConfigs],
    config: LogWriterConfig
  ): LogWriterConfig = {
    context.getRuntime.unique(config.funcName)
    context.getRuntime.logWriters.append(config)
    config
  }
}

trait LogWriterConfigs extends ServiceConfigs {

  val logWriters: mutable.ArrayBuffer[LogWriterConfig] = mutable.ArrayBuffer[LogWriterConfig]()

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]
    val tc = builder.typeContext()

    logWriters foreach { mc =>
      val p = com.twitter.botmaker.runtime.LogWriterOperator(mc)
      p.toCompile.foreach(u => builder.addCompilerUnit(u))
    }

    builder
  }
}
