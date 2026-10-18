package com.twitter.simclusters_v2.scalding.embedding.tfg

import com.twitter.dal.client.dataset.KeyValDALDataset
import com.twitter.scalding._
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.hdfs_sources.EntityEmbeddingsSources
import com.twitter.simclusters_v2.thriftscala.EmbeddingType
import com.twitter.simclusters_v2.thriftscala.ModelVersion
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingId
import com.twitter.simclusters_v2.thriftscala.UserToInterestedInClusterScores
import com.twitter.simclusters_v2.thriftscala.{SimClustersEmbedding => ThriftSimClustersEmbedding}
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp

object FavInferredLanguageTfgBasedTopicEmbeddingsAdhocApp
    extends InferredLanguageTfgBasedTopicEmbeddingsBaseApp
    with AdhocExecutionApp {
  override val isAdhoc: Boolean = true
  override val embeddingType: EmbeddingType = EmbeddingType.FavInferredLanguageTfgTopic
  override val embeddingSource: KeyValDALDataset[
    KeyVal[SimClustersEmbeddingId, ThriftSimClustersEmbedding]
  ] = EntityEmbeddingsSources.FavInferredLanguageTfgTopicEmbeddingsDataset
  override val pathSuffix: String = "fav_inferred_lang_tfg_topic_embeddings"
  override val modelVersion: ModelVersion = ModelVersion.Model20m145kUpdated
  override def scoreExtractor: UserToInterestedInClusterScores => Double = scores =>
    scores.favScore.getOrElse(0.0)
}

object FavInferredLanguageTfgBasedTopicEmbeddingsScheduledApp
    extends InferredLanguageTfgBasedTopicEmbeddingsBaseApp
    with ScheduledExecutionApp {
  override val isAdhoc: Boolean = false
  override val embeddingType: EmbeddingType = EmbeddingType.FavInferredLanguageTfgTopic
  override val embeddingSource: KeyValDALDataset[
    KeyVal[SimClustersEmbeddingId, ThriftSimClustersEmbedding]
  ] = EntityEmbeddingsSources.FavInferredLanguageTfgTopicEmbeddingsDataset
  override val pathSuffix: String = "fav_inferred_lang_tfg_topic_embeddings"
  override val modelVersion: ModelVersion = ModelVersion.Model20m145kUpdated
  override def scoreExtractor: UserToInterestedInClusterScores => Double = scores =>
    scores.favScore.getOrElse(0.0)

  override val firstTime: RichDate = RichDate("2020-07-04")
  override val batchIncrement: Duration = Days(1)
}
