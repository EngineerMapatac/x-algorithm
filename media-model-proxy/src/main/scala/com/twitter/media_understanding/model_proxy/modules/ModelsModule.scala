package com.twitter.media_understanding.model_proxy.modules

import com.google.inject.Provides
import com.google.inject.Singleton
import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.finagle.thrift.ClientId
import com.twitter.inject.TwitterModule
import com.twitter.media_understanding.model_proxy.clients.ModelClient
import com.twitter.media_understanding.model_proxy.model_descriptors._
import com.twitter.media_understanding.model_proxy.services.ModelsService

object ModelsModule extends TwitterModule {
  @Provides
  @Singleton
  def providesModelsService(
    clientId: ClientId,
    statsReceiver: StatsReceiver,
    registry: ModelDescriptorRegistry,
    serviceIdentifier: ServiceIdentifier,
  ): ModelsService = {
    new ModelsService(
      buildClients(clientId, statsReceiver, registry, serviceIdentifier),
      statsReceiver.scope("DeepbirdService"),
      registry,
    )
  }

  private def buildClients(
    clientId: ClientId,
    receiver: StatsReceiver,
    registry: ModelDescriptorRegistry,
    serviceIdentifier: ServiceIdentifier,
  ): Map[MediaModel, List[ModelClient]] = {
    registry
      .getModels()
      .map { model =>
        val descriptor = registry.getDescriptor(model).get
        val builders = descriptor.serviceBuilders()
        (
          model,
          builders.map(_.build(clientId, serviceIdentifier))
        )
      }.toMap
  }
}
