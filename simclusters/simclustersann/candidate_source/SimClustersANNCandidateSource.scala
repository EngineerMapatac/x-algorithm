package com.twitter.simclustersann.candidate_source

import com.twitter.finagle.stats.StatsReceiver
import com.twitter.frigate.common.base.Stats
import com.twitter.simclusters_v2.common.ClusterId
import com.twitter.simclusters_v2.thriftscala.SimplifiedClusterDetails
import com.twitter.simclusters_v2.common.SimClustersEmbedding
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingId
import com.twitter.simclustersann.thriftscala.SimClustersANNConfig
import com.twitter.simclustersann.thriftscala.SimClustersANNTweetCandidate
import com.twitter.storehaus.ReadableStore
import com.twitter.util.Future

case class SimClustersANNCandidateSource(
  approximateCosineSimilarity: ApproximateCosineSimilarity,
  clusterTweetCandidatesStore: ReadableStore[ClusterId, Seq[(TweetId, Double, Double)]],
  simClustersEmbeddingStore: ReadableStore[SimClustersEmbeddingId, SimClustersEmbedding],
  clusterDetailsStore: ReadableStore[String, SimplifiedClusterDetails],
  statsReceiver: StatsReceiver) {
  private val stats = statsReceiver.scope(this.getClass.getName)
  private val fetchSourceEmbeddingStat = stats.scope("fetchSourceEmbedding")
  private val fetchCandidatesStat = stats.scope("fetchCandidates")
  private val candidateScoresStat = stats.stat("candidateScoresMap")
  private val embeddingScoreDistributionStat = stats.stat("embeddingScoreDistribution")

  def get(
    query: SimClustersANNCandidateSource.Query
  ): Future[Option[Seq[SimClustersANNTweetCandidate]]] = {

    val sourceEmbeddingId = query.sourceEmbeddingId
    val config = query.config
    for {
      maybeSimClustersEmbedding <- Stats.track(fetchSourceEmbeddingStat) {
        simClustersEmbeddingStore.get(query.sourceEmbeddingId)
      }
      maybeFilteredCandidates <- maybeSimClustersEmbedding match {
        case Some(sourceEmbedding) =>
          sourceEmbedding.embedding.foreach {
            case (clusterId, score) =>
              embeddingScoreDistributionStat.add(score.toFloat)
          }
          for {
            candidates <- Stats.trackSeq(fetchCandidatesStat) {
              fetchCandidates(sourceEmbeddingId, sourceEmbedding, config)
            }
          } yield {
            fetchCandidatesStat
              .stat(sourceEmbeddingId.embeddingType.name, sourceEmbeddingId.modelVersion.name).add(
                candidates.size)
            Some(candidates)
          }
        case None =>
          fetchCandidatesStat
            .stat(sourceEmbeddingId.embeddingType.name, sourceEmbeddingId.modelVersion.name).add(0)
          Future.None
      }
    } yield {
      maybeFilteredCandidates
    }
  }

  private def fetchCandidates(
    sourceEmbeddingId: SimClustersEmbeddingId,
    sourceEmbedding: SimClustersEmbedding,
    config: SimClustersANNConfig
  ): Future[Seq[SimClustersANNTweetCandidate]] = {
    val clusterIds =
      sourceEmbedding
        .truncate(config.maxScanClusters).getClusterIds()
        .toSet

    for {
      clusterTweetsMap <- Future.collect {
        clusterTweetCandidatesStore.multiGet(clusterIds)
      }
      clusterDetailsMap <- Future.collect {
        clusterDetailsStore.multiGet(clusterIds.map(_.toString))
      }
    } yield {
      approximateCosineSimilarity(
        sourceEmbedding = sourceEmbedding,
        sourceEmbeddingId = sourceEmbeddingId,
        config = config,
        candidateScoresStat = (i: Int) => candidateScoresStat.add(i),
        clusterTweetsMap = clusterTweetsMap,
        clusterDetailsMap = clusterDetailsMap
      ).map {
        case (tweetId, score) =>
          SimClustersANNTweetCandidate(
            tweetId = tweetId,
            score = score
          )
      }
    }
  }
}

object SimClustersANNCandidateSource {
  case class Query(
    sourceEmbeddingId: SimClustersEmbeddingId,
    config: SimClustersANNConfig)
}
