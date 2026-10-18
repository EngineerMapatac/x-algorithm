package com.twitter.botmaker.runtime.config

import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.runtime.thriftjava.ThriftTypeConfig

object ThriftType extends ConfigUnit[ServiceConfigs, ThriftTypeConfig] {

  override def description: String = "Define a thrift type."

  override def apply(
    context: Context[ServiceConfigs],
    config: ThriftTypeConfig
  ): ThriftTypeConfig = {
    context.getRuntime.addThriftType(config)
    config
  }
}
