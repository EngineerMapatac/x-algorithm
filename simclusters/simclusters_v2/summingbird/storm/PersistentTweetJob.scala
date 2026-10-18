package com.twitter.simclusters_v2.summingbird.storm

import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.summingbird.common.Implicits
import com.twitter.simclusters_v2.summingbird.common.Monoids.PersistentSimClustersEmbeddingLongestL2NormMonoid
import com.twitter.simclusters_v2.summingbird.common.StatsUtil
import com.twitter.simclusters_v2.summingbird.stores.PersistentTweetEmbeddingStore.LatestEmbeddingVersion
import com.twitter.simclusters_v2.summingbird.stores.PersistentTweetEmbeddingStore.LongestL2EmbeddingVersion
import com.twitter.simclusters_v2.summingbird.stores.PersistentTweetEmbeddingStore.PersistentTweetEmbeddingId
import com.twitter.simclusters_v2.thriftscala.PersistentSimClustersEmbedding
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbedding
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingMetadata
import com.twitter.summingbird.option.JobId
import com.twitter.summingbird.Platform
import com.twitter.summingbird.Producer
import com.twitter.summingbird.TailProducer
import com.twitter.timelineservice.thriftscala.Event
import com.twitter.tweetypie.thriftscala.StatusCounts

private[storm] object PersistentTweetJob {
  import StatsUtil._

  private val MinFavoriteCount = 8
  type Timestamp = Long

  val longestL2NormMonoid = new PersistentSimClustersEmbeddingLongestL2NormMonoid()

  def generate[P <: Platform[P]](
    timelineEventSource: Producer[P, Event],
    tweetStatusCountService: P#Service[TweetId, StatusCounts],
    tweetEmbeddingService: P#Service[TweetId, SimClustersEmbedding],
    persistentTweetEmbeddingStoreWithLatestAggregation: P#Store[
      PersistentTweetEmbeddingId,
      PersistentSimClustersEmbedding
    ],
    persistentTweetEmbeddingStoreWithLongestL2NormAggregation: P#Store[
      PersistentTweetEmbeddingId,
      PersistentSimClustersEmbedding
    ]
  )(
    implicit jobId: JobId
  ): TailProducer[P, Any] = {

    val timelineEvents: Producer[P, (TweetId, Timestamp)] = timelineEventSource
      .collect {
        case Event.Favorite(favoriteEvent) =>
          (favoriteEvent.tweetId, favoriteEvent.eventTimeMs)
      }

    val filteredEvents = timelineEvents
      .leftJoin[StatusCounts](tweetStatusCountService)
      .filter {
        case (_, (_, Some(statusCounts))) =>
          statusCounts.favoriteCount.exists(_ >= MinFavoriteCount)
        case _ =>
          false
      }
      .leftJoin[SimClustersEmbedding](tweetEmbeddingService)

    val latestAndPersistentEmbeddingProducer = filteredEvents
      .collect {
        case (tweetId, ((eventTimeMs, _), Some(tweetEmbedding))) =>
          (
            PersistentTweetEmbeddingId(tweetId, timestampInMs = LatestEmbeddingVersion),
            PersistentSimClustersEmbedding(
              tweetEmbedding,
              SimClustersEmbeddingMetadata(updatedAtMs = Some(eventTimeMs), updatedCount = Some(1))
            ))
      }
      .observe("num_of_embedding_updates")
      .sumByKey(persistentTweetEmbeddingStoreWithLatestAggregation)(
        Implicits.persistentSimClustersEmbeddingMonoid)
      .name("latest_embedding_producer")
      .flatMap {
        case (persistentTweetEmbeddingId, (maybeEmbedding, deltaEmbedding)) =>
          lastQualifiedUpdatedCount(
            maybeEmbedding.flatMap(_.metadata.updatedCount),
            deltaEmbedding.metadata.updatedCount
          ).map { newUpdateCount =>
            (
              persistentTweetEmbeddingId.copy(timestampInMs =
                deltaEmbedding.metadata.updatedAtMs.getOrElse(0L)),
              deltaEmbedding.copy(metadata =
                deltaEmbedding.metadata.copy(updatedCount = Some(newUpdateCount)))
            )
          }
      }
      .observe("num_of_extra_embedding")
      .sumByKey(persistentTweetEmbeddingStoreWithLatestAggregation)(
        Implicits.persistentSimClustersEmbeddingMonoid)
      .name("persistent_embeddings_producer")

    val longestL2NormEmbeddingProducer = filteredEvents
      .collect {
        case (tweetId, ((eventTimeMs, Some(statusCounts)), Some(tweetEmbedding))) =>
          (
            PersistentTweetEmbeddingId(tweetId, timestampInMs = LongestL2EmbeddingVersion),
            PersistentSimClustersEmbedding(
              tweetEmbedding,
              SimClustersEmbeddingMetadata(
                updatedAtMs = Some(eventTimeMs),
                updatedCount = statusCounts.favoriteCount.map(_ + 1)
              )
            ))
      }
      .observe("num_longest_l2_norm_updates")
      .sumByKey(persistentTweetEmbeddingStoreWithLongestL2NormAggregation)(longestL2NormMonoid)
      .name("longest_l2_norm_embedding_producer")

    latestAndPersistentEmbeddingProducer.also(longestL2NormEmbeddingProducer)
  }

  private def lastQualifiedUpdatedCount(
    existingUpdateCount: Option[Long],
    deltaUpdateCount: Option[Long]
  ): Option[Int] = {
    val existing = existingUpdateCount.getOrElse(0L)
    val sum = existing + deltaUpdateCount.getOrElse(0L)
    qualifiedSet.filter { i => (existing < i) && (i <= sum) }.lastOption
  }

  private lazy val qualifiedSet = 3
    .until(25).map { i => Math.pow(2, i).toInt }.toSet

}
