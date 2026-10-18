package com.twitter.simclusters_v2.summingbird.stores

import com.twitter.frigate.common.store.strato.StratoFetchableStore
import com.twitter.frigate.common.store.strato.StratoStore
import com.twitter.ml.api.thriftscala.Embedding
import com.twitter.ml.api.util.BufferToIterators.RichFloatBuffer
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.common.VersionId
import com.twitter.simclusters_v2.thriftscala.PersistentTwhinUserEmbedding
import com.twitter.simclusters_v2.thriftscala.TwhinTweetEmbedding
import com.twitter.storehaus.ReadableStore
import com.twitter.storehaus.Store
import com.twitter.strato.client.Client
import java.nio.ByteBuffer
import java.nio.ByteOrder

object TwhinUserEmbeddingStore {

  val prodStratoColumn = "ml/featureStore/twhinUserEngagementEmbeddingFsv1.User"
  val prodRealTimeUserStratoColumn = "recommendations/twhin/twhinUserRealTimeEmbeddings"
  val prodUserNegativeStratoColumn = "recommendations/twhin/twhinUserNegativeEmbeddings"

  val prodRebuildUserColumn = "ml/featureStore/twhinRebuildUserEngagement.User"
  val refreshedUserColumn = "recommendations/twhin/CachedTwhinRebuildUserEngagementEmbeddings.User"
  val prodRebuildUserRealTimePositiveStratoColumn =
    "recommendations/twhin/twhinRebuildUserRealtimePositiveEmbeddings"
  val prodRebuildUserRealTimeNegativeStratoColumn =
    "recommendations/twhin/twhinRebuildUserRealtimeNegativeEmbeddings"

  def userEmbeddingStore(
    stratoClient: Client,
    column: String
  ): ReadableStore[UserId, TwhinTweetEmbedding] = {
    StratoFetchableStore
      .withUnitView[UserId, ByteBuffer](stratoClient, column)
      .mapValues { byteBuffer =>
        val floatBuffer = skipEmbeddingBBHeader(byteBuffer)
          .slice()
          .order(ByteOrder.LITTLE_ENDIAN)
          .asFloatBuffer

        val array = new Array[Float](floatBuffer.remaining())
        floatBuffer.get(array)
        val embedding = array.toSeq.map(_.toDouble)

        TwhinTweetEmbedding(embedding = embedding)
      }
  }

  def userRebuildEmbeddingStore(
    stratoClient: Client,
    column: String
  ): ReadableStore[UserId, TwhinTweetEmbedding] = {
    StratoFetchableStore
      .withUnitView[UserId, Embedding](stratoClient, column)
      .mapValues { embedding: Embedding =>
        embedding.tensor
          .map(tensor =>
            TwhinTweetEmbedding(tensor.content
              .order(ByteOrder.LITTLE_ENDIAN).asFloatBuffer.iterator.map(_.toDouble).toList)).get
      }
  }

  def persistentRealTimeUserEmbeddingStore(
    stratoClient: Client,
    column: String = prodRealTimeUserStratoColumn
  ): Store[UserId, PersistentTwhinUserEmbedding] = {
    StratoStore
      .withUnitView[(UserId, Long), PersistentTwhinUserEmbedding](stratoClient, column)
      .composeKeyMapping(userKey)
  }

  def persistentVersionedRealTimeUserEmbeddingStore(
    stratoClient: Client,
    column: String
  ): Store[(UserId, VersionId), PersistentTwhinUserEmbedding] = {
    StratoStore
      .withUnitView[(UserId, VersionId), PersistentTwhinUserEmbedding](
        stratoClient,
        column).composeKeyMapping(userVersionedKey)
  }

  private def userKey(key: UserId): (UserId, Long) = {
    (key, 0L)
  }

  private def userVersionedKey(key: (UserId, VersionId)): (UserId, VersionId) = {
    key
  }

  private def skipEmbeddingBBHeader(bb: ByteBuffer): ByteBuffer = {
    val bb_copy = bb.duplicate()
    bb_copy.getLong()
    bb_copy
  }
}
