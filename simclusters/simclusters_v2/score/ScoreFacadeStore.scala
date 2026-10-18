package com.twitter.simclusters_v2.score

import com.twitter.finagle.stats.BroadcastStatsReceiver
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.hermit.store.common.ObservedReadableStore
import com.twitter.simclusters_v2.thriftscala.ScoringAlgorithm
import com.twitter.simclusters_v2.thriftscala.{ScoreId => ThriftScoreId}
import com.twitter.simclusters_v2.thriftscala.{Score => ThriftScore}
import com.twitter.storehaus.ReadableStore
import com.twitter.util.Future

class ScoreFacadeStore private (
  stores: Map[ScoringAlgorithm, ReadableStore[ThriftScoreId, ThriftScore]])
    extends ReadableStore[ThriftScoreId, ThriftScore] {

  override def get(k: ThriftScoreId): Future[Option[ThriftScore]] = {
    findStore(k).get(k)
  }

  override def multiGet[K1 <: ThriftScoreId](ks: Set[K1]): Map[K1, Future[Option[ThriftScore]]] = {
    if (ks.isEmpty) {
      Map.empty
    } else {
      val head = ks.head
      val notSameType = ks.exists(k => k.algorithm != head.algorithm)
      if (!notSameType) {
        findStore(head).multiGet(ks)
      } else {
        ks.groupBy(id => id.algorithm).flatMap {
          case (_, ks) =>
            findStore(ks.head).multiGet(ks)
        }
      }
    }
  }

  private def findStore(id: ThriftScoreId): ReadableStore[ThriftScoreId, ThriftScore] = {
    stores.get(id.algorithm) match {
      case Some(store) => store
      case None =>
        throw new IllegalArgumentException(s"The Scoring Algorithm ${id.algorithm} doesn't exist.")
    }
  }

}

object ScoreFacadeStore {
  def buildWithMetrics(
    readableStores: Map[ScoringAlgorithm, ReadableStore[ThriftScoreId, ThriftScore]],
    aggregatedStores: Map[ScoringAlgorithm, AggregatedScoreStore],
    statsReceiver: StatsReceiver
  ) = {
    val scopedStatsReceiver = statsReceiver.scope("score_facade_store")

    def wrapStore(
      scoringAlgorithm: ScoringAlgorithm,
      store: ReadableStore[ThriftScoreId, ThriftScore]
    ): ReadableStore[ThriftScoreId, ThriftScore] = {
      val sr = BroadcastStatsReceiver(
        Seq(
          scopedStatsReceiver.scope("all"),
          scopedStatsReceiver.scope(scoringAlgorithm.name)
        ))
      ObservedReadableStore(store)(sr)
    }

    val stores = (readableStores ++ aggregatedStores).map {
      case (algo, store) => algo -> wrapStore(algo, store)
    }
    val store = new ScoreFacadeStore(stores = stores)

    assert(
      readableStores.keySet.forall(algorithm => !aggregatedStores.keySet.contains(algorithm)),
      "Keys for stores are disjoint")

    aggregatedStores.values.foreach(_.set(store))

    store
  }

}
