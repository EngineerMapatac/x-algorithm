package com.twitter.simclusters_v2.common

object SimClustersEmbeddingAttentionUtil {

  def softmax(scores: Seq[Double]): Seq[Double] = {
    val maxScore = scores.max
    val expScores = scores.map(s => Math.exp(s - maxScore))
    val sumExpScores = expScores.sum
    expScores.map(_ / sumExpScores)
  }

  def similarityVector(
    key: SimClustersEmbedding,
    values: Seq[SimClustersEmbedding]
  ): Seq[Double] = {
    values.map { value =>
      key.cosineSimilarity(value)
    }
  }

  def weightedSum(
    embeddings: Seq[SimClustersEmbedding],
    weights: Seq[Double]
  ): SimClustersEmbedding = {
    assert(embeddings.size == weights.size)

    SimClustersEmbedding.sum(
      embeddings.zipWithIndex.map {
        case (embedding, i) =>
          embedding.scalarMultiply(weights(i))
      }
    )
  }

  def selfAttention(
    key: SimClustersEmbedding,
    values: Seq[SimClustersEmbedding]
  ): SimClustersEmbedding = {
    weightedSum(values, softmax(similarityVector(key, values)))
  }

  def selfAttentions(embeddings: Seq[SimClustersEmbedding]): Seq[SimClustersEmbedding] = {
    embeddings.map { embedding =>
      selfAttention(embedding, embeddings)
    }
  }

  def crossAttentions(
    keys: Seq[SimClustersEmbedding],
    query: SimClustersEmbedding
  ): Double = {
    selfAttention(query, keys).cosineSimilarity(query)
  }

}
