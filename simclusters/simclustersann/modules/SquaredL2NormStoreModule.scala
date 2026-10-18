package com.twitter.simclustersann.modules

import com.google.inject.Provides
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.inject.TwitterModule
import com.twitter.product_mixer.shared_library.memcached_client.MemcachedClientBuilder
import com.twitter.simclusters_v2.thriftscala.EntityWithVersion
import com.twitter.simclusters_v2.thriftscala.SquaredL2Norm
import com.twitter.storehaus.Store
import com.twitter.storehaus_internal.memcache.MemcacheStore
import com.twitter.bijection.Injection
import com.twitter.bijection.scrooge.CompactScalaCodec
import com.twitter.conversions.DurationOps._
import com.twitter.finagle.memcached.compressing.scheme.Lz4
import com.twitter.hashing.KeyHasher
import javax.inject.Singleton

object SquaredL2NormStoreModule extends TwitterModule {
  private val ScopeName = ""
  private val ProdDestName = "/srv#/prod/local/cache/simclusters_core_alt:twemcaches"
  private val StagingDestName = "/srv#/prod/local/cache/simclusters_core_alt_light:twemcaches"
  private val TTL_DURATION = 2.days

  @Singleton
  @Provides
  def providesSquaredL2NormMemcachedStore(
    serviceIdentifier: ServiceIdentifier,
    statsReceiver: StatsReceiver
  ): Store[EntityWithVersion, SquaredL2Norm] = {

    val destName = serviceIdentifier.environment.toLowerCase match {
      case "prod" => ProdDestName
      case _ => StagingDestName
    }
    implicit val keyInjection: Injection[EntityWithVersion, Array[Byte]] =
      CompactScalaCodec(EntityWithVersion)
    implicit val valueInjection: Injection[SquaredL2Norm, Array[Byte]] =
      CompactScalaCodec(SquaredL2Norm)

    val client = MemcachedClientBuilder.buildMemcachedClient(
      destName = destName,
      numTries = 2,
      numConnections = 1,
      requestTimeout = 2.seconds,
      globalTimeout = 2.seconds,
      connectTimeout = 2.seconds,
      acquisitionTimeout = 2.seconds,
      serviceIdentifier = serviceIdentifier,
      statsReceiver = statsReceiver.scope(ScopeName),
      compressionScheme = Lz4,
      keyHasher = Some(KeyHasher.KETAMA)
    )

    MemcacheStore.store[EntityWithVersion, SquaredL2Norm](
      keyPrefix = "etk_l2_",
      client = client,
      ttl = TTL_DURATION,
      flag = 0
    )
  }
}
