package com.twitter.simclusters_v2.score

import com.twitter.simclusters_v2.thriftscala.{ScoreId => ThriftScoreId}
import com.twitter.simclusters_v2.thriftscala.{Score => ThriftScore}
import com.twitter.storehaus.ReadableStore

trait AggregatedScoreStore extends ReadableStore[ThriftScoreId, ThriftScore] {

  protected var scoreFacadeStore: ReadableStore[ThriftScoreId, ThriftScore] = ReadableStore.empty

  private[score] def set(facadeStore: ReadableStore[ThriftScoreId, ThriftScore]): Unit = {
    this.synchronized {
      scoreFacadeStore = facadeStore
    }
  }
}
