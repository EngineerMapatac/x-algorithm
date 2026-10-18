package com.twitter.simclusters_v2.summingbird.stores

import com.twitter.frigate.common.store.strato.StratoFetchableStore
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.storehaus.ReadableStore
import com.twitter.strato.client.Client
import com.twitter.strato.thrift.ScroogeConvImplicits._
import com.twitter.tweetypie.thriftscala.StatusCounts

object TweetStatusCountsStore {

  def tweetStatusCountsStore(
    stratoClient: Client,
    column: String
  ): ReadableStore[TweetId, StatusCounts] = {
    StratoFetchableStore
      .withView[TweetId, Unit, Long](stratoClient, column, ())
      .mapValues(toStatusCounts)
  }

  private def toStatusCounts(favCount: Long): StatusCounts =
    StatusCounts(
      favoriteCount = Some(favCount),
      retweetCount = Some(0L),
      replyCount = Some(0L),
      quoteCount = Some(0L)
    )
}
