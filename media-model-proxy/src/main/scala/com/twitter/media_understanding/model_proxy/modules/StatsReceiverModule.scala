package com.twitter.media_understanding.model_proxy.modules

import com.google.inject.Provides
import com.twitter.finagle.stats.LoadedStatsReceiver
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.inject.TwitterModule
import javax.inject.Singleton

object StatsReceiverModule extends TwitterModule {

  @Provides
  @Singleton
  def providesStatsReceiver(): StatsReceiver = {
    LoadedStatsReceiver.scope("cortex-media-annotator")
  }
}
