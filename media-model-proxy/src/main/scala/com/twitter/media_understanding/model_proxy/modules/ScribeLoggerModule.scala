package com.twitter.media_understanding.model_proxy.modules

import com.google.inject.Provides
import com.google.inject.Singleton
import com.twitter.decider.Decider
import com.twitter.finagle.stats.NullStatsReceiver
import com.twitter.inject.TwitterModule
import com.twitter.logging.Logger
import com.twitter.media_understanding.model_proxy.config.Config
import com.twitter.media_understanding.model_proxy.util.NullScriber
import com.twitter.media_understanding.model_proxy.util.ScribeLogger
import com.twitter.media_understanding.model_proxy.util.Scriber
import com.twitter.mediaservices.commons.config.Env

object ScribeLoggerModule extends TwitterModule {
  private[this] val log = Logger.get(getClass)

  val scribeCategory = "cortex_media_annotator"

  @Provides
  @Singleton
  def providesScriber(env: Env, config: () => Config, decider: Decider): Scriber = {
    log.info("Initializing scribe client..")
    env match {
      case Env.Development => NullScriber
      case Env.Prod => new ScribeLogger(scribeCategory, config, decider, NullStatsReceiver)
      case _ => new ScribeLogger("test_".concat(scribeCategory), config, decider, NullStatsReceiver)
    }
  }
}
