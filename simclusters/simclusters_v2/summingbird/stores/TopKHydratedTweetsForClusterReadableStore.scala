package com.twitter.simclusters_v2.summingbird.stores

import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.simclusters_v2.summingbird.common.ClientConfigs
import com.twitter.simclusters_v2.summingbird.common.Implicits
import com.twitter.simclusters_v2.thriftscala.FullClusterId
import com.twitter.simclusters_v2.thriftscala.TopKHydratedTweetsWithScores
import com.twitter.storehaus.algebra.MergeableStore
import com.twitter.storehaus_internal.memcache.Memcache
import com.twitter.summingbird.batch.BatchID
import com.twitter.summingbird_internal.bijection.BatchPairImplicits
import com.twitter.simclusters_v2.summingbird.common.Implicits.topKHydratedTweetsWithScoresCodec
import com.twitter.simclusters_v2.summingbird.common.Implicits.topKHydratedTweetsWithScoresMonoid

object TopKHydratedTweetsForClusterReadableStore {

  private[summingbird] final lazy val onlineMergeableStore: (
    String,
    ServiceIdentifier
  ) => MergeableStore[(FullClusterId, BatchID), TopKHydratedTweetsWithScores] = {
    (storePath: String, serviceIdentifier: ServiceIdentifier) =>
      Memcache.getMemcacheStore[(FullClusterId, BatchID), TopKHydratedTweetsWithScores](
        ClientConfigs.clusterTopHydratedTweetsMemcacheConfig(storePath, serviceIdentifier)
      )(
        BatchPairImplicits.keyInjection[FullClusterId](Implicits.fullClusterIdCodec),
        topKHydratedTweetsWithScoresCodec,
        topKHydratedTweetsWithScoresMonoid
      )
  }
}
