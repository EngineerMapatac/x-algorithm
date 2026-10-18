package com.twitter.simclusters_v2.tweet_similarity

import com.twitter.ml.api.Feature.Binary
import com.twitter.ml.api.Feature.Continuous
import com.twitter.ml.api.Feature.Discrete
import com.twitter.ml.api.Feature.SparseContinuous
import com.twitter.ml.api.util.FDsl._
import com.twitter.ml.api.DataRecord
import com.twitter.ml.api.FeatureContext
import com.twitter.ml.api.IRecordOneToOneAdapter
import com.twitter.ml.featurestore.catalog.features.recommendations.ProducerSimClustersEmbedding
import com.twitter.ml.featurestore.lib.UserId
import com.twitter.ml.featurestore.lib.data.PredictionRecord
import com.twitter.ml.featurestore.lib.data.PredictionRecordAdapter
import com.twitter.ml.featurestore.lib.entity.Entity
import com.twitter.ml.featurestore.lib.feature.BoundFeatureSet

object TweetSimilarityFeatures {
  val QueryTweetId = new Discrete("query_tweet.id")
  val CandidateTweetId = new Discrete("candidate_tweet.id")
  val QueryTweetEmbedding = new SparseContinuous("query_tweet.simclusters_embedding")
  val CandidateTweetEmbedding = new SparseContinuous("candidate_tweet.simclusters_embedding")
  val QueryTweetEmbeddingNorm = new Continuous("query_tweet.embedding_norm")
  val CandidateTweetEmbeddingNorm = new Continuous("candidate_tweet.embedding_norm")
  val QueryTweetTimestamp = new Discrete("query_tweet.timestamp")
  val CandidateTweetTimestamp = new Discrete("candidate_tweet.timestamp")
  val TweetPairCount = new Discrete("popularity_count.tweet_pair")
  val QueryTweetCount = new Discrete("popularity_count.query_tweet")
  val CosineSimilarity = new Continuous("meta.cosine_similarity")
  val Label = new Binary("co-engagement.label")

  val FeatureContext: FeatureContext = new FeatureContext(
    QueryTweetId,
    CandidateTweetId,
    QueryTweetEmbedding,
    CandidateTweetEmbedding,
    QueryTweetEmbeddingNorm,
    CandidateTweetEmbeddingNorm,
    QueryTweetTimestamp,
    CandidateTweetTimestamp,
    TweetPairCount,
    QueryTweetCount,
    CosineSimilarity,
    Label
  )

  def isCoengaged(dataRecord: DataRecord): Boolean = {
    dataRecord.getFeatureValue(Label)
  }
}

class TweetSimilarityFeaturesStoreConfig(identifier: String) {
  val bindingIdentifier: Entity[UserId] = Entity[UserId](identifier)

  val featureStoreBoundFeatureSet: BoundFeatureSet = BoundFeatureSet(
    ProducerSimClustersEmbedding.FavBasedEmbedding20m145kUpdated.bind(bindingIdentifier))

  val predictionRecordAdapter: IRecordOneToOneAdapter[PredictionRecord] =
    PredictionRecordAdapter.oneToOne(featureStoreBoundFeatureSet)
}
