package com.twitter.simclusters_v2.summingbird.common

import com.twitter.conversions.DurationOps._
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.thrift.ClientId
import com.twitter.storehaus_internal.memcache.ConnectionConfig
import com.twitter.storehaus_internal.memcache.MemcacheConfig
import com.twitter.storehaus_internal.util.KeyPrefix
import com.twitter.storehaus_internal.util.TTL
import com.twitter.strato.client.Strato
import com.twitter.strato.client.{Client => StratoClient}
import com.twitter.storehaus_internal.memcache.ConnectionConfig
import com.twitter.storehaus_internal.memcache.MemcacheStore
import com.twitter.finagle.memcached.{Client => MemcachedClient}
import com.twitter.finagle.Memcached
import com.twitter.finagle.memcached.protocol.Command
import com.twitter.finagle.memcached.protocol.Response
import com.twitter.finagle.memcached.compressing.scheme.Lz4
import com.twitter.storehaus_internal.memcache.MemcacheConfig
import com.twitter.finagle.param.HighResTimer
import com.twitter.finagle.service.RetryExceptionsFilter
import com.twitter.finagle.service.RetryPolicy
import com.twitter.finagle.service.TimeoutFilter
import com.twitter.finagle.stats.DefaultStatsReceiver
import com.twitter.finagle.GlobalRequestTimeoutException
import com.twitter.finagle.mtls.client.MtlsStackClient._
import com.twitter.finagle.Resolver
import com.twitter.hashing.KeyHasher

object ClientConfigs {

  com.twitter.server.Init()

  final lazy val simClustersCoreAltCachePath =
    "/srv#/prod/local/cache/simclusters_core_alt"

  final lazy val simClustersCoreAltLightCachePath =
    "/srv#/prod/local/cache/simclusters_core_alt_light"

  final lazy val simClustersCoreAltVideoCachePath =
    "/srv#/prod/local/cache/simclusters_core_alt_video"

  final lazy val develSimClustersCoreCachePath =
    "/srv#/test/local/cache/twemcache_simclusters_core"

  final lazy val logFavBasedTweet20M145K2020StratoPath =
    "recommendations/simclusters_v2/embeddings/logFavBasedTweet20M145K2020Persistent"

  final lazy val logFavBasedTweet20M145K2020UncachedStratoPath =
    "recommendations/simclusters_v2/embeddings/logFavBasedTweet20M145K2020-UNCACHED"

  final lazy val develLogFavBasedTweet20M145K2020StratoPath =
    "recommendations/simclusters_v2/embeddings/logFavBasedTweet20M145K2020Devel"

  final lazy val entityClusterScoreMemcacheConfig: (String, ServiceIdentifier) => MemcacheConfig = {
    (path: String, serviceIdentifier: ServiceIdentifier) =>
      new MemcacheConfig {
        val connectionConfig: ConnectionConfig =
          ConnectionConfig(path, serviceIdentifier = serviceIdentifier)
        override val keyPrefix: KeyPrefix = KeyPrefix(s"ecs_")
        override val ttl: TTL = TTL(8.hours)
      }
  }

  final lazy val tweetSquaredL2NormMemcacheConfig: (String, ServiceIdentifier) => MemcacheConfig = {
    (path: String, serviceIdentifier: ServiceIdentifier) =>
      new MemcacheConfig {
        val connectionConfig: ConnectionConfig =
          ConnectionConfig(path, serviceIdentifier = serviceIdentifier)
        override val keyPrefix: KeyPrefix = KeyPrefix(s"etk_l2_")
        override val ttl: TTL = TTL(2.days)
      }
  }

  final lazy val tweetTopKClustersMemcacheConfig: (String, ServiceIdentifier) => MemcacheConfig = {
    (path: String, serviceIdentifier: ServiceIdentifier) =>
      new MemcacheConfig {
        val connectionConfig: ConnectionConfig =
          ConnectionConfig(path, serviceIdentifier = serviceIdentifier)
        override val keyPrefix: KeyPrefix = KeyPrefix(s"etk_")
        override val ttl: TTL = TTL(2.days)
      }
  }

  final lazy val clusterTopTweetsMemcacheConfig: (String, ServiceIdentifier) => MemcacheConfig = {
    (path: String, serviceIdentifier: ServiceIdentifier) =>
      new MemcacheConfig {
        val connectionConfig: ConnectionConfig =
          ConnectionConfig(path, serviceIdentifier = serviceIdentifier)
        override val keyPrefix: KeyPrefix = KeyPrefix(s"ctkt_")
        override val ttl: TTL = TTL(8.hours)
      }
  }

  final lazy val clusterTopHydratedTweetsMemcacheConfig: (
    String,
    ServiceIdentifier
  ) => MemcacheConfig = { (path: String, serviceIdentifier: ServiceIdentifier) =>
    new MemcacheConfig {
      val connectionConfig: ConnectionConfig =
        ConnectionConfig(path, serviceIdentifier = serviceIdentifier)
      override val keyPrefix: KeyPrefix = KeyPrefix(s"ctkht_")
      override val ttl: TTL = TTL(8.hours)
    }
  }

  final lazy val stratoClient: ServiceIdentifier => StratoClient = { serviceIdentifier =>
    Strato.client
      .withRequestTimeout(2.seconds)
      .withMutualTls(serviceIdentifier)
      .build()
  }

  final lazy val timer = HighResTimer.Default
  final lazy val timeoutFilter = {
    val duration = 2.seconds
    val exc = new GlobalRequestTimeoutException(duration)
    new TimeoutFilter[Command, Response](duration, exc, timer)
  }

  final lazy val retryFilter = new RetryExceptionsFilter[Command, Response](
    retryPolicy = RetryPolicy.tries(
      2,
      RetryPolicy.TimeoutAndWriteExceptionsOnly
        .orElse(RetryPolicy.ChannelClosedExceptionsOnly)
    ),
    timer = HighResTimer.Default,
    statsReceiver = DefaultStatsReceiver
  )

  final lazy val memcachedClient: ServiceIdentifier => MemcachedClient = { serviceIdentifier =>
    Memcached.client
      .withMutualTls(serviceIdentifier)
      .withKeyHasher(KeyHasher.KETAMA)
      .withSession
      .acquisitionTimeout(3.seconds)
      .withRequestTimeout(3.seconds)
      .filtered(timeoutFilter.andThen(retryFilter))
      .withCompressionScheme(Lz4)
      .newRichClient(
        dest = Resolver.eval("/srv#/prod/local/cache/simclusters_core_alt:twemcaches"),
        label = "TweetSquaredL2NormMemcache"
      )
  }

  private final lazy val thriftClientId: String => ClientId = { env: String =>
    ClientId(s"simclusters_v2_summingbird.$env")
  }

}
