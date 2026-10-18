package com.twitter.simclusters_v2.common

import com.twitter.simclusters_v2.common.SimClustersMultiEmbeddingId._
import com.twitter.simclusters_v2.thriftscala.SimClustersMultiEmbedding.Ids
import com.twitter.simclusters_v2.thriftscala.SimClustersMultiEmbedding.Values
import com.twitter.simclusters_v2.thriftscala.SimClustersMultiEmbedding
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingId
import com.twitter.simclusters_v2.thriftscala.SimClustersMultiEmbeddingId

object SimClustersMultiEmbedding {

  def toSimClustersEmbeddingIdWithScores(
    simClustersMultiEmbeddingId: SimClustersMultiEmbeddingId,
    simClustersMultiEmbedding: SimClustersMultiEmbedding
  ): Seq[(SimClustersEmbeddingId, Double)] = {
    simClustersMultiEmbedding match {
      case Values(values) =>
        values.embeddings.zipWithIndex.map {
          case (embeddingWithScore, i) =>
            (toEmbeddingId(simClustersMultiEmbeddingId, i), embeddingWithScore.score)
        }
      case Ids(ids) =>
        ids.ids.map(_.toTuple)
    }
  }

}
