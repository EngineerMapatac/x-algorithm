package com.twitter.botmaker.runtime

import com.twitter.botmaker.runtime.config.MemCachedEndpointConfigs
import com.twitter.botmaker.runtime.thriftjava.MemCachedEndpointConfig
import com.twitter.finagle.factory.TimeoutFactory
import com.twitter.finagle.liveness.FailureAccrualFactory
import com.twitter.finagle.Memcached
import com.twitter.finagle.Resolver
import com.twitter.finagle.memcached
import com.twitter.finagle.memcached.MockClient
import com.twitter.finagle.service.TimeoutFilter
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util.Duration
import com.twitter.util.Future
import com.twitter.util.Time

case class MemCachedEndpointRegistry(config: MemCachedEndpointConfig, endpoint: memcached.Client)
    extends ServiceRegistry[MemCachedEndpointConfig] {

  override val name: String = config.endpoint

  override def close(deadline: Time): Future[Unit] = endpoint.close(deadline)
}

trait MemCachedEndpointFactory extends TwitterRuntime {

  def build(config: MemCachedEndpointConfig, scoped: StatsReceiver): memcached.Client = {
    if (config.servicePath == "/srv#/test/local/cache/test") {
      return new MockClient()
    }
    Memcached.client
      .withStatsReceiver(scoped.scope(config.endpoint))
      .configured(TimeoutFilter.Param(Duration.fromMilliseconds(config.timeoutFilterMilis)))
      .configured(TimeoutFactory.Param(Duration.fromMilliseconds(config.timeoutFactoryMilis)))
      .configured(
        FailureAccrualFactory.Param(
          config.numFailuresFailureAccrual,
          () => Duration.fromMilliseconds(config.markDeadForMilisFailuresFailureAccrual)
        )
      )
      .newRichClient(
        dest = Resolver.eval(config.servicePath),
        label = config.label
      )
  }
}

trait MemCachedEndpointContainer extends ServiceContainer with MemCachedEndpointFactory {
  container =>

  override type _CONFIGS_TYPE <: MemCachedEndpointConfigs

  private def isReusable(config: MemCachedEndpointConfig) =
    isReusable[MemCachedEndpointConfig, MemCachedEndpointRegistry](config.endpoint, config)

  def memCachedEndpoints(endpoint: String): memcached.Client = {
    serviceRegistry[MemCachedEndpointRegistry](endpoint).get.endpoint
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val memCachedEndpoints = configs.getMemCachedEndpoints.map {
      case config if isReusable(config) =>
        logger.info(s"reuse MemCached Client ${config.endpoint}")
        serviceRegistry[MemCachedEndpointRegistry](config.endpoint).get
      case config =>
        logger.info(s"create MemCached Client ${config.endpoint}")
        val endpoint = build(config, scoped.scope("MemCachedEndpoint"))
        MemCachedEndpointRegistry(config, endpoint)
    }
    memCachedEndpoints ++ super.build(configs, scoped)
  }
}
