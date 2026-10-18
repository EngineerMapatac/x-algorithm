package com.twitter.simclusters_v2.summingbird.stores

import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.simclusters_v2.summingbird.common.Implicits.clustersWithScoreMonoid
import com.twitter.simclusters_v2.summingbird.common.Implicits.clustersWithScoresCodec
import com.twitter.storehaus.algebra.MergeableStore
import com.twitter.simclusters_v2.summingbird.common.ClientConfigs
import com.twitter.simclusters_v2.summingbird.common.Implicits
import com.twitter.simclusters_v2.thriftscala.ClustersWithScores
import com.twitter.simclusters_v2.thriftscala.FullClusterIdBucket
import com.twitter.simclusters_v2.thriftscala.SimClusterEntity
import com.twitter.storehaus_internal.memcache.Memcache
import com.twitter.summingbird.batch.BatchID
import com.twitter.summingbird_internal.bijection.BatchPairImplicits

object EntityClusterScoreReadableStore {

  private[simclusters_v2] final lazy val onlineMergeableStore: (
    String,
    ServiceIdentifier
  ) => MergeableStore[
    ((SimClusterEntity, FullClusterIdBucket), BatchID),
    ClustersWithScores
  ] = { (path: String, serviceIdentifier: ServiceIdentifier) =>
    Memcache
      .getMemcacheStore[((SimClusterEntity, FullClusterIdBucket), BatchID), ClustersWithScores](
        ClientConfigs.entityClusterScoreMemcacheConfig(path, serviceIdentifier)
      )(
        BatchPairImplicits.keyInjection[(SimClusterEntity, FullClusterIdBucket)](
          Implicits.entityWithClusterInjection
        ),
        clustersWithScoresCodec,
        clustersWithScoreMonoid
      )
  }

}
