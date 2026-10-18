package com.twitter.simclusters_v2.summingbird.stores

import com.twitter.frigate.common.store.strato.StratoFetchableStore
import com.twitter.frigate.common.store.strato.StratoStore
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.common.VersionId
import com.twitter.simclusters_v2.thriftscala.PersistentTwhinTweetEmbedding
import com.twitter.simclusters_v2.thriftscala.TwhinTweetEmbedding
import com.twitter.storehaus.ReadableStore
import com.twitter.storehaus.Store
import com.twitter.strato.client.Client

object TwhinTweetEmbeddingStore {

  val prodPersistentTweetStratoColumn = "recommendations/twhin/twhinTweetEmbeddings"
  val prodPersistentVideoStratoColumn = "recommendations/twhin/twhinVideoEmbeddings"
  val prodCachedTweetStratoColumn = "recommendations/twhin/CachedTwhinTweetEmbeddings.Tweet"
  val prodCachedVideoStratoColumn = "recommendations/twhin/CachedTwhinVideoEmbeddings.Tweet"

  val prodPersistentRebuildTweetStratoColumn = "recommendations/twhin/twhinRebuildTweetEmbeddings"
  val prodCachedRebuildTweetStratoColumn =
    "recommendations/twhin/CachedTwhinRebuildVersionedTweetEmbeddings"

  def persistentTweetEmbeddingStore(
    stratoClient: Client,
    column: String = prodPersistentTweetStratoColumn
  ): Store[TweetId, PersistentTwhinTweetEmbedding] = {
    StratoStore
      .withUnitView[(TweetId, Long), PersistentTwhinTweetEmbedding](stratoClient, column)
      .composeKeyMapping(tweetKey)
  }

  def persistentVersionedTweetEmbeddingStore(
    stratoClient: Client,
    column: String = prodPersistentRebuildTweetStratoColumn
  ): Store[(TweetId, VersionId), PersistentTwhinTweetEmbedding] = {
    StratoStore
      .withUnitView[(TweetId, VersionId), PersistentTwhinTweetEmbedding](stratoClient, column)
      .composeKeyMapping(tweetVersionedKey)
  }

  def cachedTweetEmbeddingStore(
    stratoClient: Client,
    column: String = prodCachedTweetStratoColumn
  ): ReadableStore[TweetId, TwhinTweetEmbedding] = {
    StratoFetchableStore
      .withUnitView[TweetId, TwhinTweetEmbedding](stratoClient, column)
  }

  def cachedVersionedTweetEmbeddingStore(
    stratoClient: Client,
    column: String = prodCachedRebuildTweetStratoColumn
  ): ReadableStore[(TweetId, VersionId), TwhinTweetEmbedding] = {
    StratoFetchableStore
      .withUnitView[(TweetId, VersionId), TwhinTweetEmbedding](stratoClient, column)
  }

  private def tweetKey(key: TweetId): (TweetId, Long) = {
    (key, 0L)
  }

  private def tweetVersionedKey(key: (TweetId, VersionId)): (TweetId, VersionId) = {
    key
  }
}
