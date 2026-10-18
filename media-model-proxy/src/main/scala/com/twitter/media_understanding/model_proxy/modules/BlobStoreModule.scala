package com.twitter.media_understanding.model_proxy.modules

import com.google.inject.Provides
import com.google.inject.Singleton
import com.twitter.conversions.DurationOps._
import com.twitter.conversions.StorageUnitOps._
import com.twitter.cortex_common.services.client.{ClientBuilder => CortexClientBuilder}
import com.twitter.finagle.builder.ClientBuilder
import com.twitter.finagle.client.DefaultPool
import com.twitter.finagle.mtls.authentication.EmptyServiceIdentifier
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.mtls.client.MtlsStackClient._
import com.twitter.finagle.stats.NullStatsReceiver
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.finagle.thrift.ClientId
import com.twitter.finagle.tracing.DefaultTracer
import com.twitter.finagle.ChannelClosedException
import com.twitter.finagle.Http
import com.twitter.finagle.IndividualRequestTimeoutException
import com.twitter.finagle.WriteException
import com.twitter.inject.TwitterModule
import com.twitter.logging.Logger
import com.twitter.media_understanding.model_proxy.config.Tunables
import com.twitter.media_understanding.model_proxy.services.BlobStoreService
import com.twitter.mediaservices.commons.config.Env
import com.twitter.mediaservices.commons.BlobStoreClient
import com.twitter.mediaservices.commons.DestinationUtils
import com.twitter.mediaservices.commons.ShouldRetryBuilder

object BlobStoreModule extends TwitterModule {
  private[this] val log = Logger.get(getClass)
  val serviceName = "BlobstoreService"

  @Provides
  @Singleton
  def providesBlobStoreService(
    clientId: ClientId,
    statsReceiver: StatsReceiver,
    serviceIdentifier: ServiceIdentifier,
  ): BlobStoreService = {
    new BlobStoreService(
      blobStore(clientId, statsReceiver, serviceIdentifier),
      statsReceiver.scope(serviceName))
  }

  private val shouldRetryBuilder: ShouldRetryBuilder = {
    ShouldRetryBuilder()
      .add(classOf[WriteException])
      .add(classOf[ChannelClosedException])
      .add(classOf[IndividualRequestTimeoutException], duration = 10.seconds, maxRetries = 3)
  }

  private def configureFinagleClient(statsReceiver: StatsReceiver) = {
    val cb = ClientBuilder()
      .name(serviceName)
      .reportTo(statsReceiver)
      .reportHostStats(NullStatsReceiver)
      .retryPolicy(CortexClientBuilder.mkRetryPolicy(2))
      .tcpConnectTimeout(50.milliseconds)
      .hostConnectionLimit(100)
      .tracer(DefaultTracer)

    val maxWaiters = cb
      .params[DefaultPool.Param]
      .copy(maxWaiters = 1000)

    cb.configured(maxWaiters)
  }

  private def blobStore(
    clientId: ClientId,
    statsReceiver: StatsReceiver,
    serviceIdentifier: ServiceIdentifier
  ): BlobStoreClient = {
    log.info("Initializing blobstore client ...")
    val endpoint = if (serviceIdentifier == EmptyServiceIdentifier) "internal" else "internal_https"
    val destination =
      DestinationUtils.newWilyDest("blobstore", "frontend", endpointName = Some(endpoint))
    val httpStack = Http.client
      .withMaxResponseSize(64.megabytes)
      .withNoHttp2
      .withMutualTls(serviceIdentifier)

    val builder = configureFinagleClient(statsReceiver)
      .stack(httpStack)
      .hostConnectionLimit(100)
      .requestTimeout(8.seconds)

    BlobStoreClient(
      clientId.name,
      statsReceiver.scope(serviceName),
      builder,
      destination,
      Env.Prod,
      Tunables.blobStoreMaxExtraLoad,
      enforceGdprComplianceForTtlObjects = true,
      failPartialStreams = false
    )
  }
}
