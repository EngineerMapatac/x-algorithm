package com.twitter.botmaker.runtime.config

import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.runtime.ConfigFailure
import com.twitter.botmaker.runtime.thriftjava.RegexConfig

object RegexOptions extends ConfigUnit[RegexOptionConfigs, RegexConfig] {

  override def description: String = "Configure regex options."

  override def apply(context: Context[RegexOptionConfigs], config: RegexConfig): RegexConfig = {
    context.getRuntime.addRegexOptions(config)
    config
  }
}

trait RegexOptionConfigs extends ServiceConfigs {

  private final var regexOptions = new RegexConfig()
    .setTimeoutMilis(1000L)
    .setMaximumPendingRequests(20L)

  final def getRegexOptions: RegexConfig = regexOptions

  final def addRegexOptions(config: RegexConfig): Unit = {

    if (config.maximumPendingRequests <= 0) {
      throw ConfigFailure(
        s"Regex maximumPendingRequests must be greater than 0"
      )
    }

    if (config.timeoutMilis <= 0) {
      throw ConfigFailure(
        s"Regex timeoutMilis must be greater than 0"
      )
    }

    if (config.maximumCacheItems <= 0) {
      throw ConfigFailure(
        s"Regex maximumCacheItems must be greater than 0"
      )
    }

    if (config.cacheExpirationMillis <= 0) {
      throw ConfigFailure(
        s"Regex cacheExpirationMillis must be greater than 0"
      )
    }

    regexOptions = config
  }
}
