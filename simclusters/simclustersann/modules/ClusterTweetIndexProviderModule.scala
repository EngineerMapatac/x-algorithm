package com.twitter.simclustersann.modules

import com.google.inject.Provides
import com.twitter.conversions.DurationOps._
import com.twitter.decider.Decider
import com.twitter.finagle.memcached.Client
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.hermit.store.common.ObservedCachedReadableStore
import com.twitter.hermit.store.common.ObservedMemcachedReadableStore
import com.twitter.inject.TwitterModule
import com.twitter.inject.annotations.Flag
import com.twitter.relevance_platform.common.injection.LZ4Injection
import com.twitter.relevance_platform.common.injection.SeqObjectInjection
import com.twitter.relevance_platform.simclustersann.multicluster.ClusterConfig
import com.twitter.relevance_platform.simclustersann.multicluster.ClusterTweetIndexStoreConfig
import com.twitter.simclusters_v2.common.ClusterId
import com.twitter.simclusters_v2.common.ModelVersions
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.summingbird.stores.ClusterKey
import com.twitter.simclusters_v2.summingbird.stores.TopKTweetsForClusterKeyReadableStore
import com.twitter.simclusters_v2.thriftscala.EmbeddingType
import com.twitter.simclusters_v2.thriftscala.EntityWithVersion
import com.twitter.simclusters_v2.thriftscala.SimClusterEntity
import com.twitter.simclusters_v2.thriftscala.SquaredL2Norm
import com.twitter.simclustersann.common.FlagNames
import com.twitter.storehaus.ReadableStore
import com.twitter.storehaus.Store
import com.twitter.util.Future

import javax.inject.Singleton

object ClusterTweetIndexProviderModule extends TwitterModule {

  @Singleton
  @Provides
  def providesClusterTweetIndex(
    @Flag(FlagNames.MaxTopTweetPerCluster) maxTopTweetPerCluster: Int,
    @Flag(FlagNames.CacheAsyncUpdate) asyncUpdate: Boolean,
    clusterConfig: ClusterConfig,
    serviceIdentifier: ServiceIdentifier,
    stats: StatsReceiver,
    decider: Decider,
    simClustersANNCacheClient: Client,
    squaredL2NormStore: Store[EntityWithVersion, SquaredL2Norm]
  ): ReadableStore[ClusterId, Seq[(TweetId, Double, Double)]] = {
    val topTweetsForClusterStore = clusterConfig.clusterTweetIndexStoreConfig match {
      case manhattanConfig: ClusterTweetIndexStoreConfig.Manhattan =>
        TopKTweetsForClusterKeyReadableStore.getClusterToTopKTweetsStoreFromManhattanRO(
          maxTopTweetPerCluster,
          manhattanConfig,
          serviceIdentifier)
      case memCacheConfig: ClusterTweetIndexStoreConfig.Memcached =>
        TopKTweetsForClusterKeyReadableStore.getClusterToTopKTweetsStoreFromMemCache(
          maxTopTweetPerCluster,
          memCacheConfig,
          serviceIdentifier)
      case _ =>
        ReadableStore.empty
    }

    val embeddingType: EmbeddingType = clusterConfig.candidateTweetEmbeddingType
    val modelVersion: String = ModelVersions.toKnownForModelVersion(clusterConfig.modelVersion)

    val store: ReadableStore[ClusterId, Seq[(TweetId, Double, Double)]] =
      topTweetsForClusterStore
        .composeKeyMapping { id: ClusterId =>
          ClusterKey(id, modelVersion, embeddingType)
        }.flatMapValues { tweetIdAndScoresSeq =>
          Future.collect(
            tweetIdAndScoresSeq.map {
              case (tweetId, score) =>
                val entityWithVersion =
                  EntityWithVersion(SimClusterEntity.TweetId(tweetId), clusterConfig.modelVersion)

                squaredL2NormStore
                  .get(entityWithVersion)
                  .handle {
                    case _: Exception =>
                      None
                  }
                  .map { optNorm =>
                    val favSquaredL2Norm = optNorm.flatMap(_.favSquaredL2Norm).getOrElse(0.0)
                    (tweetId, score, favSquaredL2Norm)
                  }
            }
          )
        }

    val memcachedTopTweetsForClusterStore =
      ObservedMemcachedReadableStore.fromCacheClient(
        backingStore = store,
        cacheClient = simClustersANNCacheClient,
        ttl = 15.minutes,
        asyncUpdate = asyncUpdate
      )(
        valueInjection = LZ4Injection.compose(SeqObjectInjection[(Long, Double, Double)]()),
        statsReceiver = stats.scope("cluster_tweet_index_mem_cache"),
        keyToString = { k =>
          s"scz:c2t:l2:${k}_${embeddingType}_${modelVersion}_$maxTopTweetPerCluster"
        }
      )

    val cachedStore: ReadableStore[ClusterId, Seq[(TweetId, Double, Double)]] = {
      ObservedCachedReadableStore.from[ClusterId, Seq[(TweetId, Double, Double)]](
        memcachedTopTweetsForClusterStore,
        ttl = 10.minute,
        maxKeys = 150000,
        cacheName = "cluster_tweet_index_cache",
        windowSize = 10000L
      )(stats.scope("cluster_tweet_index_store"))
    }
    cachedStore
  }
}
