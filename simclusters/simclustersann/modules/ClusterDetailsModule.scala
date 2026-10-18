package com.twitter.simclustersann.modules

import com.google.inject.Provides
import com.twitter.bijection.Bufferable
import com.twitter.bijection.Injection
import com.twitter.bijection.scrooge.CompactScalaCodec
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.inject.TwitterModule
import com.twitter.simclusters_v2.thriftscala.ClusterDetails
import com.twitter.simclusters_v2.thriftscala.SimplifiedClusterDetails
import com.twitter.storage.client.manhattan.kv.ManhattanKVClientMtlsParams
import com.twitter.storehaus.ReadableStore
import com.twitter.storehaus_internal.manhattan.Athena
import com.twitter.storehaus_internal.manhattan.ManhattanRO
import com.twitter.storehaus_internal.manhattan.ManhattanROConfig
import com.twitter.storehaus_internal.util.ApplicationID
import com.twitter.storehaus_internal.util.DatasetName
import com.twitter.storehaus_internal.util.HDFSPath
import com.twitter.util.Duration
import com.twitter.util.Future
import javax.inject.Singleton
import com.twitter.storehaus_internal.manhattan.Athena
import com.twitter.storehaus_internal.manhattan.ManhattanRO
import com.twitter.storehaus_internal.manhattan.ManhattanROConfig
import com.twitter.storehaus_internal.manhattan.ManhattanRODefaults
import com.twitter.hermit.store.common.ObservedCachedReadableStore
import com.twitter.hermit.store.common.ObservedReadableStore
import com.twitter.storehaus_internal.util._

object ClusterDetailsModule extends TwitterModule {

  @Provides
  @Singleton
  def providesClusterDetailsStore(
    serviceIdentifier: ServiceIdentifier,
    statsReceiver: StatsReceiver
  ): ReadableStore[String, SimplifiedClusterDetails] = {
    implicit val keyInjection: Injection[(String, Int), Array[Byte]] =
      Bufferable.injectionOf[(String, Int)]
    implicit val valueInjection: Injection[ClusterDetails, Array[Byte]] =
      CompactScalaCodec(ClusterDetails)

    val modelName = "20M_145K_2020"

    val clusterDetailsManhattanROConfig: ManhattanROConfig =
      new ManhattanROConfig {
        val hdfsPath = HDFSPath("")
        val applicationID = ApplicationID("simclusters_v2")
        val datasetName = DatasetName("simclusters_v2_cluster_details_20m_145k_2020")
        val cluster = Athena
        override val manhattanTimeout: ManhattanTimeout =
          ManhattanTimeout(ManhattanRODefaults.DEFAULT_MANHATTAN_TIMEOUT)
        override val connectTimeout: ConnectTimeout = ConnectTimeout(
          ManhattanRODefaults.DEFAULT_CONNECT_TIMEOUT
        )
        override val requestTimeout: RequestTimeout = RequestTimeout(
          ManhattanRODefaults.DEFAULT_REQUEST_TIMEOUT
        )
        override val overallTimeout: OverallTimeout = OverallTimeout(
          ManhattanRODefaults.DEFAULT_OVERALL_TIMEOUT
        )
        override val retries: Retries = Retries(ManhattanRODefaults.DEFAULT_RETRIES)
      }

    val clusterDetailsOriginalStore: ReadableStore[(String, Int), ClusterDetails] = {
      val mtlsParams = ManhattanKVClientMtlsParams(
        serviceIdentifier = serviceIdentifier
      )
      ManhattanRO.getReadableStoreWithMtls[(String, Int), ClusterDetails](
        clusterDetailsManhattanROConfig,
        mtlsParams
      )
    }

    val mappedOriginalStore: ReadableStore[String, ClusterDetails] =
      clusterDetailsOriginalStore.composeKeyMapping(clusterIdString =>
        (modelName, clusterIdString.toInt))

    def valueFn(clusterDetails: ClusterDetails): Future[SimplifiedClusterDetails] =
      Future.value(
        SimplifiedClusterDetails(
          clusterDetails.numUsersWithAnyNonZeroScore,
          clusterDetails.numUsersWithNonZeroFollowScore,
          clusterDetails.numUsersWithNonZeroFavScore
        )
      )

    val transformedStore: ReadableStore[String, SimplifiedClusterDetails] =
      ReadableStore.convert(mappedOriginalStore) { identity[String] }(valueFn)

    val observedStore =
      ObservedReadableStore.apply(transformedStore)(statsReceiver.scope("cluster_details_store"))

    ObservedCachedReadableStore.from(
      store = observedStore,
      ttl = Duration.fromHours(10),
      maxKeys = 500000,
      cacheName = "cluster_details_cache",
      windowSize = 10000L
    )(statsReceiver.scope("cluster_details_cached_store"))
  }
}
