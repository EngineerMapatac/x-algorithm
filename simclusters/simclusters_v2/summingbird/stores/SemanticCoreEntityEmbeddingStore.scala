package com.twitter.simclusters_v2.summingbird.stores

import com.twitter.frigate.common.store.strato.StratoStore
import com.twitter.simclusters_v2.common.ModelVersions
import com.twitter.simclusters_v2.common.ModelVersions._
import com.twitter.simclusters_v2.thriftscala.EmbeddingType
import com.twitter.simclusters_v2.thriftscala.InternalId
import com.twitter.simclusters_v2.thriftscala.LocaleEntityId
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingId
import com.twitter.simclusters_v2.thriftscala.{SimClustersEmbedding => ThriftSimClustersEmbedding}
import com.twitter.storehaus.ReadableStore
import com.twitter.strato.client.Client
import com.twitter.strato.thrift.ScroogeConvImplicits._
import com.twitter.simclusters_v2.common.SimClustersEmbedding

object SemanticCoreEntityEmbeddingStore {

  private val column =
    "recommendations/simclusters_v2/embeddings/semanticCoreEntityPerLanguageEmbeddings20M145KUpdated"

  private def getDefaultStore(
    stratoClient: Client
  ): ReadableStore[SimClustersEmbeddingId, ThriftSimClustersEmbedding] = {
    StratoStore
      .withUnitView[SimClustersEmbeddingId, ThriftSimClustersEmbedding](stratoClient, column)
  }

  def getFavBasedLocaleEntityEmbeddingStore(
    stratoClient: Client
  ): ReadableStore[LocaleEntityId, SimClustersEmbedding] = {
    getDefaultStore(stratoClient)
      .composeKeyMapping[LocaleEntityId] { entityId =>
        SimClustersEmbeddingId(
          EmbeddingType.FavBasedSematicCoreEntity,
          ModelVersions.Model20M145KUpdated,
          InternalId.LocaleEntityId(entityId)
        )
      }
      .mapValues(SimClustersEmbedding(_))
  }
}
