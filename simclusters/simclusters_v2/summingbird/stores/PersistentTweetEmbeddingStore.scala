package com.twitter.simclusters_v2.summingbird.stores

import com.twitter.finagle.stats.StatsReceiver
import com.twitter.frigate.common.store.strato.StratoFetchableStore
import com.twitter.frigate.common.store.strato.StratoStore
import com.twitter.simclusters_v2.common.SimClustersEmbedding
import com.twitter.simclusters_v2.common.SimClustersEmbedding._
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.thriftscala.PersistentSimClustersEmbedding
import com.twitter.storage.client.manhattan.kv.ManhattanKVClientMtlsParams
import com.twitter.storehaus.ReadableStore
import com.twitter.storehaus.Store
import com.twitter.strato.catalog.Scan.Slice
import com.twitter.strato.client.Client
import com.twitter.strato.thrift.ScroogeConvImplicits._

object PersistentTweetEmbeddingStore {

  val LogFavBasedColumn =
    "recommendations/simclusters_v2/embeddings/logFavBasedTweet20M145KUpdatedPersistent"
  val LogFavBasedColumn20m145k2020 =
    "recommendations/simclusters_v2/embeddings/logFavBasedTweet20M145K2020Persistent"

  val LogFavBased20m145k2020Dataset = "log_fav_based_tweet_20m_145k_2020_embeddings"
  val LogFavBased20m145kUpdatedDataset = "log_fav_based_tweet_20m_145k_updated_embeddings"

  val DefaultMaxLength = 15

  def mostRecentTweetEmbeddingStore(
    stratoClient: Client,
    column: String,
    maxLength: Int = DefaultMaxLength
  ): ReadableStore[TweetId, SimClustersEmbedding] = {
    StratoFetchableStore
      .withUnitView[(TweetId, Timestamp), PersistentSimClustersEmbedding](stratoClient, column)
      .composeKeyMapping[TweetId]((_, LatestEmbeddingVersion))
      .mapValues(_.embedding.truncate(maxLength))
  }

  def longestL2NormTweetEmbeddingStore(
    stratoClient: Client,
    column: String
  ): ReadableStore[TweetId, SimClustersEmbedding] =
    StratoFetchableStore
      .withUnitView[(TweetId, Timestamp), PersistentSimClustersEmbedding](stratoClient, column)
      .composeKeyMapping[TweetId]((_, LongestL2EmbeddingVersion))
      .mapValues(_.embedding)

  def mostRecentTweetEmbeddingStoreManhattan(
    mhMtlsParams: ManhattanKVClientMtlsParams,
    dataset: String,
    statsReceiver: StatsReceiver,
    maxLength: Int = DefaultMaxLength
  ): ReadableStore[TweetId, SimClustersEmbedding] =
    ManhattanFromStratoStore
      .createPersistentTweetStore(
        dataset = dataset,
        mhMtlsParams = mhMtlsParams,
        statsReceiver = statsReceiver
      ).composeKeyMapping[TweetId]((_, LatestEmbeddingVersion))
      .mapValues[SimClustersEmbedding](_.embedding.truncate(maxLength))

  def longestL2NormTweetEmbeddingStoreManhattan(
    mhMtlsParams: ManhattanKVClientMtlsParams,
    dataset: String,
    statsReceiver: StatsReceiver,
    maxLength: Int = 50
  ): ReadableStore[TweetId, SimClustersEmbedding] =
    ManhattanFromStratoStore
      .createPersistentTweetStore(
        dataset = dataset,
        mhMtlsParams = mhMtlsParams,
        statsReceiver = statsReceiver
      ).composeKeyMapping[TweetId]((_, LongestL2EmbeddingVersion))
      .mapValues[SimClustersEmbedding](_.embedding.truncate(maxLength))

  private[simclusters_v2] def persistentTweetEmbeddingStore(
    stratoClient: Client,
    column: String
  ): Store[PersistentTweetEmbeddingId, PersistentSimClustersEmbedding] = {
    StratoStore
      .withUnitView[(TweetId, Timestamp), PersistentSimClustersEmbedding](stratoClient, column)
      .composeKeyMapping(_.toTuple)
  }

  type Timestamp = Long

  case class PersistentTweetEmbeddingId(
    tweetId: TweetId,
    timestampInMs: Timestamp = LatestEmbeddingVersion) {
    lazy val toTuple: (TweetId, Timestamp) = (tweetId, timestampInMs)
  }

  private[summingbird] val LatestEmbeddingVersion = 0L
  private[summingbird] val LongestL2EmbeddingVersion = 1L

  private[stores] val DefaultSlice = Slice[Long](from = None, to = None, limit = None)
}
