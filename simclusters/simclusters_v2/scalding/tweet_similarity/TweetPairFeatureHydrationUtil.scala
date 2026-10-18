package com.twitter.simclusters_v2.scalding.tweet_similarity

import com.twitter.ml.api.util.FDsl._
import com.twitter.ml.api.DataRecord
import com.twitter.ml.api.DataRecordMerger
import com.twitter.ml.api.DataSetPipe
import com.twitter.ml.api.FeatureContext
import com.twitter.ml.featurestore.lib.data.EntityIds.Entry
import com.twitter.ml.featurestore.lib.data.EntityIds
import com.twitter.ml.featurestore.lib.data.FeatureValuesById
import com.twitter.ml.featurestore.lib.data.PredictionRecord
import com.twitter.scalding.typed.TypedPipe
import com.twitter.simclusters_v2.common.SimClustersEmbedding._
import com.twitter.simclusters_v2.tweet_similarity.ModelBasedTweetSimilaritySimClustersEmbeddingAdapter.NormalizedCandidateEmbAdapter
import com.twitter.simclusters_v2.tweet_similarity.ModelBasedTweetSimilaritySimClustersEmbeddingAdapter.NormalizedQueryEmbAdapter
import com.twitter.simclusters_v2.tweet_similarity.TweetSimilarityFeatures
import com.twitter.simclusters_v2.tweet_similarity.TweetSimilarityFeaturesStoreConfig
import com.twitter.simclusters_v2.common.Timestamp
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.scalding.tweet_similarity.TweetPairLabelCollectionUtil.FeaturedTweet
import com.twitter.simclusters_v2.thriftscala.PersistentSimClustersEmbedding
import com.twitter.simclusters_v2.thriftscala.{SimClustersEmbedding => ThriftSimClustersEmbedding}

object TweetPairFeatureHydrationUtil {
  val QueryTweetConfig = new TweetSimilarityFeaturesStoreConfig("query_tweet_user_id")
  val CandidateTweetConfig = new TweetSimilarityFeaturesStoreConfig("candidate_tweet_user_id")
  val DataRecordMerger = new DataRecordMerger()

  def extractEmbeddings(
    persistentEmbeddings: TypedPipe[((TweetId, Timestamp), PersistentSimClustersEmbedding)]
  ): TypedPipe[(TweetId, (Timestamp, ThriftSimClustersEmbedding))] = {
    persistentEmbeddings
      .collect {
        case ((tweetId, _), embedding) if embedding.metadata.updatedAtMs.isDefined =>
          (tweetId, (embedding.metadata.updatedAtMs.get, embedding.embedding))
      }
  }

  def getTweetPairsWithPersistentEmbeddings(
    tweetPairs: TypedPipe[(FeaturedTweet, FeaturedTweet, Boolean)],
    persistentEmbeddings: TypedPipe[((TweetId, Timestamp), PersistentSimClustersEmbedding)]
  ): TypedPipe[(FeaturedTweet, FeaturedTweet, Boolean)] = {
    val extractedEmbeddings = extractEmbeddings(persistentEmbeddings)
    tweetPairs
      .groupBy {
        case (queryFeaturedTweet, _, _) => queryFeaturedTweet.tweet
      }
      .join(extractedEmbeddings)
      .collect {
        case (
              _,
              (
                (queryFeaturedTweet, candidateFeaturedTweet, label),
                (embeddingTimestamp, embedding)))
            if embeddingTimestamp <= queryFeaturedTweet.timestamp =>
          ((queryFeaturedTweet, candidateFeaturedTweet), (embeddingTimestamp, embedding, label))
      }
      .group
      .maxBy(_._1)
      .map {
        case ((queryFeaturedTweet, candidateFeaturedTweet), (_, embedding, label)) =>
          (
            candidateFeaturedTweet.tweet,
            (queryFeaturedTweet.copy(embedding = Some(embedding)), candidateFeaturedTweet, label)
          )
      }
      .join(extractedEmbeddings)
      .collect {
        case (
              _,
              (
                (queryFeaturedTweet, candidateFeaturedTweet, label),
                (embeddingTimestamp, embedding)))
            if embeddingTimestamp <= candidateFeaturedTweet.timestamp =>
          ((queryFeaturedTweet, candidateFeaturedTweet), (embeddingTimestamp, embedding, label))
      }
      .group
      .maxBy(_._1)
      .map {
        case ((queryFeaturedTweet, candidateFeaturedTweet), (_, embedding, label)) =>
          (queryFeaturedTweet, candidateFeaturedTweet.copy(embedding = Some(embedding)), label)
      }
  }

  def getTweetPairsWithAuthors(
    tweetPairs: TypedPipe[(FeaturedTweet, FeaturedTweet, Boolean)],
    tweetAuthorPairs: TypedPipe[(TweetId, UserId)]
  ): TypedPipe[(FeaturedTweet, FeaturedTweet, Boolean)] = {
    tweetPairs
      .groupBy { case (queryFeaturedTweet, _, _) => queryFeaturedTweet.tweet }
      .join(tweetAuthorPairs)
      .values
      .groupBy { case ((_, candidateFeaturedTweet, _), _) => candidateFeaturedTweet.tweet }
      .join(tweetAuthorPairs)
      .values
      .map {
        case (
              ((queryFeaturedTweet, candidateFeaturedTweet, label), queryAuthor),
              candidateAuthor) =>
          (
            queryFeaturedTweet.copy(author = Some(queryAuthor)),
            candidateFeaturedTweet.copy(author = Some(candidateAuthor)),
            label
          )
      }
  }

  def getTweetPairsWithCounts(
    tweetPairs: TypedPipe[(FeaturedTweet, FeaturedTweet, Boolean)]
  ): TypedPipe[(FeaturedTweet, FeaturedTweet, Long, Long, Boolean)] = {
    val tweetPairCount = tweetPairs.groupBy {
      case (queryFeaturedTweet, candidateFeaturedTweet, _) =>
        (queryFeaturedTweet.tweet, candidateFeaturedTweet.tweet)
    }.size

    val queryTweetCount = tweetPairs.groupBy {
      case (queryFeaturedTweet, _, _) => queryFeaturedTweet.tweet
    }.size

    tweetPairs
      .groupBy {
        case (queryFeaturedTweet, candidateFeaturedTweet, _) =>
          (queryFeaturedTweet.tweet, candidateFeaturedTweet.tweet)
      }
      .join(tweetPairCount)
      .values
      .map {
        case ((queryFeaturedTweet, candidateFeaturedTweet, label), tweetPairCount) =>
          (queryFeaturedTweet, candidateFeaturedTweet, tweetPairCount, label)
      }
      .groupBy { case (queryFeaturedTweet, _, _, _) => queryFeaturedTweet.tweet }
      .join(queryTweetCount)
      .values
      .map {
        case (
              (queryFeaturedTweet, candidateFeaturedTweet, tweetPairCount, label),
              queryTweetCount) =>
          (queryFeaturedTweet, candidateFeaturedTweet, tweetPairCount, queryTweetCount, label)
      }
  }

  def getDataSetPipeWithFeatures(
    tweetPairs: TypedPipe[(FeaturedTweet, FeaturedTweet, Boolean)],
    persistentEmbeddings: TypedPipe[((TweetId, Timestamp), PersistentSimClustersEmbedding)],
    tweetAuthorPairs: TypedPipe[(TweetId, UserId)],
    useAuthorFeatures: Boolean
  ): DataSetPipe = {
    val featuredTweetPairs =
      if (useAuthorFeatures)
        getTweetPairsWithCounts(
          getTweetPairsWithPersistentEmbeddings(
            getTweetPairsWithAuthors(tweetPairs, tweetAuthorPairs),
            persistentEmbeddings))
      else
        getTweetPairsWithCounts(
          getTweetPairsWithPersistentEmbeddings(tweetPairs, persistentEmbeddings))

    DataSetPipe(
      featuredTweetPairs.flatMap {
        case (queryFeaturedTweet, candidateFeaturedTweet, tweetPairCount, queryTweetCount, label) =>
          getDataRecordWithFeatures(
            queryFeaturedTweet,
            candidateFeaturedTweet,
            tweetPairCount,
            queryTweetCount,
            label)
      },
      FeatureContext.merge(
        TweetSimilarityFeatures.FeatureContext,
        QueryTweetConfig.predictionRecordAdapter.getFeatureContext,
        CandidateTweetConfig.predictionRecordAdapter.getFeatureContext
      )
    )
  }

  def getDataRecordWithFeatures(
    queryFeaturedTweet: FeaturedTweet,
    candidateFeaturedTweet: FeaturedTweet,
    tweetPairCount: Long,
    queryTweetCount: Long,
    label: Boolean
  ): Option[DataRecord] = {

    for {
      queryEmbedding <- queryFeaturedTweet.embedding
      candidateEmbedding <- candidateFeaturedTweet.embedding
    } yield {
      val featureDataRecord = NormalizedQueryEmbAdapter.adaptToDataRecord(queryEmbedding)
      DataRecordMerger.merge(
        featureDataRecord,
        NormalizedCandidateEmbAdapter.adaptToDataRecord(candidateEmbedding))
      featureDataRecord.setFeatureValue(
        TweetSimilarityFeatures.QueryTweetId,
        queryFeaturedTweet.tweet)
      featureDataRecord.setFeatureValue(
        TweetSimilarityFeatures.CandidateTweetId,
        candidateFeaturedTweet.tweet)
      featureDataRecord.setFeatureValue(
        TweetSimilarityFeatures.QueryTweetTimestamp,
        queryFeaturedTweet.timestamp)
      featureDataRecord.setFeatureValue(
        TweetSimilarityFeatures.CandidateTweetTimestamp,
        candidateFeaturedTweet.timestamp)
      featureDataRecord.setFeatureValue(
        TweetSimilarityFeatures.CosineSimilarity,
        queryEmbedding.cosineSimilarity(candidateEmbedding))
      featureDataRecord.setFeatureValue(TweetSimilarityFeatures.TweetPairCount, tweetPairCount)
      featureDataRecord.setFeatureValue(TweetSimilarityFeatures.QueryTweetCount, queryTweetCount)
      featureDataRecord.setFeatureValue(TweetSimilarityFeatures.Label, label)

      if (queryFeaturedTweet.author.isDefined && candidateFeaturedTweet.author.isDefined) {
        DataRecordMerger.merge(
          featureDataRecord,
          new DataRecord(
            QueryTweetConfig.predictionRecordAdapter.adaptToDataRecord(PredictionRecord(
              FeatureValuesById.empty,
              EntityIds(Entry(
                QueryTweetConfig.bindingIdentifier,
                Set(com.twitter.ml.featurestore.lib.UserId(queryFeaturedTweet.author.get))))
            )))
        )
        DataRecordMerger.merge(
          featureDataRecord,
          new DataRecord(
            CandidateTweetConfig.predictionRecordAdapter.adaptToDataRecord(PredictionRecord(
              FeatureValuesById.empty,
              EntityIds(Entry(
                CandidateTweetConfig.bindingIdentifier,
                Set(com.twitter.ml.featurestore.lib.UserId(candidateFeaturedTweet.author.get))))
            )))
        )
      }
      featureDataRecord
    }
  }
}
