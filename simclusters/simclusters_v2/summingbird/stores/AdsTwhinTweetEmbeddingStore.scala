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

object AdsTwhinTweetEmbeddingStore {

  val prodCachedRebuildTweetStratoColumn =
    "recommendations/twhin/CachedTwhinRebuildVersionedTweetEmbeddings"
  val prodPersistentRebuildTweetStratoColumn =
    "targeting/sourced_ads/prod/twhin/adsTwhinRebuildTweetEmbeddings"

  def persistentVersionedTweetEmbeddingStore(
    stratoClient: Client,
    column: String = prodPersistentRebuildTweetStratoColumn
  ): Store[(TweetId, VersionId), PersistentTwhinTweetEmbedding] = {
    StratoStore
      .withUnitView[(TweetId, VersionId), PersistentTwhinTweetEmbedding](stratoClient, column)
      .composeKeyMapping(tweetVersionedKey)
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
