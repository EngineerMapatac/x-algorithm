package com.twitter.simclusters_v2.summingbird.stores

import com.twitter.simclusters_v2.common.TweetId
import com.twitter.stitch.Stitch
import com.twitter.storehaus.FutureOps
import com.twitter.storehaus.ReadableStore
import com.twitter.strato.client.Client
import com.twitter.strato.thrift.ScroogeConvImplicits._
import com.twitter.tweetypie.thriftscala.StatusCounts
import com.twitter.util.Future

object TweetStatusCountsTesStore {

  private val FavoritesCountColumn = "tweetypie/federated/favoritesCount.Tweet"

  def tweetStatusCountsStore(
    stratoClient: Client
  ): ReadableStore[TweetId, StatusCounts] = {
    val fetcher = stratoClient.fetcher[TweetId, Unit, Long](FavoritesCountColumn)

    new ReadableStore[TweetId, StatusCounts] {
      override def get(key: TweetId): Future[Option[StatusCounts]] =
        Stitch.run(fetcher.fetch(key, ())).map(_.v.map(toStatusCounts))

      override def multiGet[K1 <: TweetId](
        keys: Set[K1]
      ): Map[K1, Future[Option[StatusCounts]]] = {
        val resultsFuture = Stitch
          .run(Stitch.traverse(keys.toSeq) { key =>
            fetcher.fetch(key, ()).map(result => key -> result.v.map(toStatusCounts))
          })
          .map(_.toMap)
        FutureOps.liftValues(keys, resultsFuture)
      }
    }
  }

  private def toStatusCounts(favCount: Long): StatusCounts =
    StatusCounts(
      favoriteCount = Some(favCount),
      retweetCount = Some(0L),
      replyCount = Some(0L),
      quoteCount = Some(0L)
    )
}
