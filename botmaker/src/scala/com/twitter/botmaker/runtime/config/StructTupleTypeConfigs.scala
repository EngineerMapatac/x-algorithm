package com.twitter.botmaker.runtime.config

import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.runtime.thriftjava.StructTupleTypeConfig

object StructTupleType extends ConfigUnit[ServiceConfigs, StructTupleTypeConfig] {

  override def description: String = "Define a StructTuple type."

  override def apply(
    context: Context[ServiceConfigs],
    config: StructTupleTypeConfig
  ): StructTupleTypeConfig = {
    context.getRuntime.addStructTupleType(config)
    config
  }
}
