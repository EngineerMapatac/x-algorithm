package com.twitter.media_understanding.model_proxy.modules

import com.google.inject.Provides
import com.google.inject.Singleton
import com.twitter.finagle.thrift.ClientId
import com.twitter.inject.TwitterModule
import com.twitter.mediaservices.commons.config.Env

object MediaModelProxyModule extends TwitterModule {

  private val environment =
    flag("environment", "development", "Environment, e.g., prod, staging, or development")

  @Provides
  @Singleton
  def env: Env = Env(environment())

  @Provides
  @Singleton
  def clientId(env: Env): ClientId = ClientId("cortex-media-annotator-server." + env.value)
}
