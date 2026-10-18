package com.twitter.simclusters_v2.summingbird.stores

import com.twitter.storehaus.WritableStore
import com.twitter.storehaus.Store
import com.twitter.storehaus_internal.memcache.ConnectionConfig
import com.twitter.storehaus_internal.memcache.MemcacheStore
import com.twitter.util.Future
import com.twitter.simclusters_v2.summingbird.common.ClientConfigs
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.simclusters_v2.thriftscala.EntityWithVersion
import com.twitter.simclusters_v2.thriftscala.SquaredL2Norm
import com.twitter.finagle.memcached.Client
import com.twitter.bijection.Injection
import com.twitter.bijection.scrooge.CompactScalaCodec
import com.twitter.conversions.DurationOps._
import com.twitter.storehaus_internal.util.KeyPrefix
import com.twitter.storehaus_internal.util.TTL
import com.twitter.finagle.Memcached
import com.twitter.finagle.memcached.protocol.Command
import com.twitter.finagle.memcached.protocol.Response
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.finagle.memcached.compressing.scheme.Lz4
import com.twitter.storehaus_internal.memcache.MemcacheConfig
import com.twitter.summingbird_internal.runner.store_config.OnlineStoreOnlyConfig
import com.twitter.logging.Logger

case class TweetSquaredL2NormWritableStore(
  underlyingStore: Store[EntityWithVersion, SquaredL2Norm])
    extends WritableStore[EntityWithVersion, SquaredL2Norm] {

  private val logger = Logger.get("TweetSquaredL2NormWritableStore")

  override def put(kv: (EntityWithVersion, SquaredL2Norm)): Future[Unit] = {
    val result = underlyingStore.put(kv._1, Some(kv._2))
    result
      .onSuccess { _ =>
        logger.info(s"Successfully put key: ${kv._1}")
      }.onFailure { e =>
        logger.error(s"Failed to put key: ${kv._1}, error: ${e.getMessage}", e)
      }
    result
  }

  override def multiPut[K1 <: EntityWithVersion](
    kvs: Map[K1, SquaredL2Norm]
  ): Map[K1, Future[Unit]] = {
    val results = underlyingStore.multiPut(kvs.mapValues(Some(_)))
    results.foreach {
      case (k, future) =>
        future
          .onSuccess { _ =>
            logger.info(s"Successfully put key in multiPut: $k")
          }.onFailure { e =>
            logger.error(s"Failed to put key in multiPut: $k, error: ${e.getMessage}", e)
          }
    }
    results
  }
}

object TweetSquaredL2NormWritableStore {
  private val TTL_DURATION = 2.days
  private val logger = Logger.get("TweetSquaredL2NormWritableStore")

  def apply(
    destName: String,
    serviceIdentifier: ServiceIdentifier
  ): WritableStore[EntityWithVersion, SquaredL2Norm] = {

    val config = new MemcacheConfig {
      override val connectionConfig =
        ConnectionConfig(destName, serviceIdentifier = serviceIdentifier)
      override val keyPrefix = KeyPrefix("etk_l2_")
      override val ttl = TTL(TTL_DURATION)
    }

    val memcachedClient = ClientConfigs.memcachedClient(serviceIdentifier)

    implicit val keyInjection: Injection[EntityWithVersion, Array[Byte]] =
      CompactScalaCodec(EntityWithVersion)
    implicit val valueInjection: Injection[SquaredL2Norm, Array[Byte]] =
      CompactScalaCodec(SquaredL2Norm)

    val store = TweetSquaredL2NormWritableStore(
      MemcacheStore.store[EntityWithVersion, SquaredL2Norm](
        config.keyPrefix.toString,
        memcachedClient
      )
    )
    logger.info("Successfully created TweetSquaredL2NormWritableStore")
    store
  }
}
