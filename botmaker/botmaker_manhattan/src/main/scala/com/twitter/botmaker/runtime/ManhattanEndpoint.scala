package com.twitter.botmaker.runtime

import com.twitter.botmaker.runtime.config.ManhattanEndpointConfigs
import com.twitter.botmaker.runtime.thriftjava.ManhattanEndpointConfig
import com.twitter.finagle.mtls.authentication.EmptyServiceIdentifier
import com.twitter.finagle.mux.transport.OpportunisticTls
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.storage.client.manhattan.kv._
import com.twitter.util.Duration
import com.twitter.util.Future
import com.twitter.util.Time

case class ManhattanEndpointRegistry(config: ManhattanEndpointConfig, endpoint: ManhattanKVEndpoint)
    extends ServiceRegistry[ManhattanEndpointConfig] {

  override val name: String = config.endpoint

  override def close(deadline: Time): Future[Unit] = {
    Future.Unit
  }
}

trait ManhattanEndpointFactory extends TwitterRuntime {
  def build(config: ManhattanEndpointConfig, scoped: StatsReceiver): ManhattanKVEndpoint = {
    val mtlsParams = if (serviceIdentifier == EmptyServiceIdentifier) {
      NoMtlsParams
    } else {
      ManhattanKVClientMtlsParams(
        serviceIdentifier = serviceIdentifier,
        opportunisticTls = OpportunisticTls.Required
      )
    }
    val client = ManhattanKVClient(
      config.appId,
      config.servicePath,
      mtlsParams
    )
    ManhattanKVEndpointBuilder(client)
      .defaultMaxTimeout(Duration.fromMilliseconds(config.defaultTimeoutMillis))
      .maxRetryCount(config.maxRetries)
      .statsReceiver(scoped.scope(config.endpoint))
      .build()
  }
}

trait ManhattanEndpointContainer extends ServiceContainer with ManhattanEndpointFactory {
  container =>

  override type _CONFIGS_TYPE <: ManhattanEndpointConfigs

  private def isReusable(config: ManhattanEndpointConfig) =
    isReusable[ManhattanEndpointConfig, ManhattanEndpointRegistry](config.endpoint, config)

  def manhattanEndpoints(endpoint: String): ManhattanKVEndpoint = {
    serviceRegistry[ManhattanEndpointRegistry](endpoint).get.endpoint
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val manhattanEndpoints = configs.getManhattanEndpoints.map {
      case config if isReusable(config) =>
        logger.info(s"reuse ManhattanKVEndpoint ${config.endpoint}")
        serviceRegistry[ManhattanEndpointRegistry](config.endpoint).get
      case config =>
        logger.info(s"create ManhattanKVEndpoint ${config.endpoint}")
        val endpoint = build(config, scoped.scope("ManhattanEndpoint"))
        ManhattanEndpointRegistry(config, endpoint)
    }
    manhattanEndpoints ++ super.build(configs, scoped)
  }
}
