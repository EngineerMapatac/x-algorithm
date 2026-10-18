package com.twitter.media_understanding.model_proxy.modules

import com.google.inject.Provides
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.inject.TwitterModule
import com.twitter.media_understanding.common.filters.LoadSheddingFilter
import com.twitter.media_understanding.model_proxy.config.Features
import com.twitter.media_understanding.model_proxy.exception.Failures
import javax.inject.Singleton

object LoadSheddingFilterModule extends TwitterModule {

  @Provides
  @Singleton
  def providesLoadSheddingFilter(
    statsReceiver: StatsReceiver,
    features: Features,
  ): LoadSheddingFilter = new LoadSheddingFilter(
    loadShedCheck = () => features.loadShedding.isAvailable,
    loadShedException = () => Failures.LoadSheddingFailure,
    statsReceiver = statsReceiver
  )
}
