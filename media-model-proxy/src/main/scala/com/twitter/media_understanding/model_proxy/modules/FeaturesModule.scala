package com.twitter.media_understanding.model_proxy.modules

import com.google.inject.Provides
import com.twitter.inject.Injector
import com.twitter.media_understanding.common.filters.AuthenticationFilter
import com.twitter.media_understanding.common.modules.DisableAdminAuthModule
import com.twitter.media_understanding.common.modules.FeaturesSecurelyModule
import com.twitter.media_understanding.common.modules.FeatureRegistryAdminModule
import com.twitter.media_understanding.model_proxy.config.Features
import com.twitter.securely._
import com.twitter.server.AdminHttpServer
import javax.inject.Singleton

case class FeaturesModule(server: AdminHttpServer)
    extends FeatureRegistryAdminModule[Features](server) {
  override val modules = Seq(
    DisableAdminAuthModule,
    FeaturesSecurelyModule
  )

  val useDevelFeatureConfig = flag[Boolean](
    "use-devel-feature-config",
    false,
    "Set the state of features as configured by default for development.",
  )

  override def authorizedGroups =
    if (DisableAdminAuthModule.disableAuth()) Set()
    else AdminGroups

  override def singletonStartup(injector: Injector) = {
    super.singletonStartup(injector)
    if (useDevelFeatureConfig()) info("Using the decider configuration for development.")
  }

  @Provides
  @Singleton
  def providesFeaturesAdminAuthFilter(
    securelyAuthenticationFilter: SecurelyAuthenticationFilter,
  ): AuthenticationFilter = {
    new AuthenticationFilter(
      DisableAdminAuthModule.disableAuth(),
      securelyAuthenticationFilter
    )
  }
}
