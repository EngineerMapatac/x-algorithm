package com.twitter.media_understanding.model_proxy.app

import com.google.inject.Module
import com.twitter.cortex_common.services.monitoring.InstanceMemoryStatsModule
import com.twitter.cortex_media_annotator.thriftscala.CortexMediaAnnotator
import com.twitter.finagle.Filter
import com.twitter.finatra.annotations.DarkTrafficFilterType
import com.twitter.finatra.decider.modules.DeciderModule
import com.twitter.finatra.mtls.thriftmux.Mtls
import com.twitter.finatra.mtls.thriftmux.filters.MtlsServerSessionTrackerFilter
import com.twitter.finatra.mtls.thriftmux.modules.MtlsThriftWebFormsModule
import com.twitter.finatra.thrift.ThriftServer
import com.twitter.finatra.thrift.filters._
import com.twitter.finatra.thrift.routing.ThriftRouter
import com.twitter.media_understanding.common.filters.LoadSheddingFilter
import com.twitter.media_understanding.model_proxy.controllers.ProxyController
import com.twitter.media_understanding.model_proxy.modules._
import com.twitter.media_understanding.model_proxy.services.ModelsService
import com.twitter.media_understanding.model_proxy.model_descriptors.ModelDescriptorRegistry
import com.twitter.mediaservices.commons.config.Env

object MediaModelProxyThriftServerMain extends MediaModelProxyThriftServer

class MediaModelProxyThriftServer extends ThriftServer with Mtls {
  override val name = "cortex-media-annotator-server"

  override val modules: Seq[Module] = Seq(
    DeciderModule,
    FeaturesModule(this),
    ConfigModule(this),
    new ThriftDiffyProxyModule,
    InstanceMemoryStatsModule,
    new MtlsThriftWebFormsModule[CortexMediaAnnotator.MethodPerEndpoint](this),
    MediaModelProxyModule,
    BlobStoreModule,
    ModelsModule,
    ScribeLoggerModule,
    LoadSheddingFilterModule,
  )

  override protected def statsReceiverModule = StatsReceiverModule

  override def postInjectorStartup(): Unit = {
    super.postInjectorStartup()

    val env = injector.instance[Env]

    if (env == Env.Staging) {
      val modelsService = injector.instance[ModelsService]
      val registry = injector.instance[ModelDescriptorRegistry]
      val validator = new ModelClientValidator(registry, modelsService)

      validator.validate()
    }
  }

  def configureThrift(router: ThriftRouter): Unit = {
    router
      .filter[LoggingMDCFilter]
      .filter[TraceIdMDCFilter]
      .filter[ThriftMDCFilter]
      .filter[AccessLoggingFilter]
      .filter[StatsFilter]
      .filter[LoadSheddingFilter]
      .filter[Filter.TypeAgnostic, DarkTrafficFilterType]
      .filter[MtlsServerSessionTrackerFilter]
      .add[ProxyController]
  }
}
