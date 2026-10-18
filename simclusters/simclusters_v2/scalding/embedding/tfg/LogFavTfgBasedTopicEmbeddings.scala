package com.twitter.simclusters_v2.scalding.embedding.tfg

import com.twitter.dal.client.dataset.KeyValDALDataset
import com.twitter.dal.client.dataset.SnapshotDALDatasetBase
import com.twitter.scalding._
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.hdfs_sources.EntityEmbeddingsSources
import com.twitter.simclusters_v2.thriftscala.EmbeddingType
import com.twitter.simclusters_v2.thriftscala.ModelVersion
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingId
import com.twitter.simclusters_v2.thriftscala.TfgTopicEmbeddings
import com.twitter.simclusters_v2.thriftscala.UserToInterestedInClusterScores
import com.twitter.simclusters_v2.thriftscala.{SimClustersEmbedding => ThriftSimClustersEmbedding}
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp

object LogFavTfgTopicEmbeddingsAdhocApp
    extends TfgBasedTopicEmbeddingsBaseApp
    with AdhocExecutionApp {
  override val isAdhoc: Boolean = true
  override val embeddingType: EmbeddingType = EmbeddingType.LogFavTfgTopic
  override val embeddingSource: KeyValDALDataset[
    KeyVal[SimClustersEmbeddingId, ThriftSimClustersEmbedding]
  ] = EntityEmbeddingsSources.LogFavTfgTopicEmbeddingsDataset
  override val pathSuffix: String = "logfav_tfg_topic_embedding"
  override val modelVersion: ModelVersion = ModelVersion.Model20m145kUpdated
  override val parquetDataSource: SnapshotDALDatasetBase[TfgTopicEmbeddings] =
    EntityEmbeddingsSources.LogFavTfgTopicEmbeddingsParquetDataset
  override def scoreExtractor: UserToInterestedInClusterScores => Double = scores =>
    scores.logFavScore.getOrElse(0.0)
}

object LogFavTfgTopicEmbeddingsScheduledApp
    extends TfgBasedTopicEmbeddingsBaseApp
    with ScheduledExecutionApp {
  override val isAdhoc: Boolean = false
  override val embeddingType: EmbeddingType = EmbeddingType.LogFavTfgTopic
  override val embeddingSource: KeyValDALDataset[
    KeyVal[SimClustersEmbeddingId, ThriftSimClustersEmbedding]
  ] = EntityEmbeddingsSources.LogFavTfgTopicEmbeddingsDataset
  override val pathSuffix: String = "logfav_tfg_topic_embedding"
  override val modelVersion: ModelVersion = ModelVersion.Model20m145kUpdated
  override def scoreExtractor: UserToInterestedInClusterScores => Double = scores =>
    scores.logFavScore.getOrElse(0.0)
  override val parquetDataSource: SnapshotDALDatasetBase[TfgTopicEmbeddings] =
    EntityEmbeddingsSources.LogFavTfgTopicEmbeddingsParquetDataset
  override val firstTime: RichDate = RichDate("2020-05-25")
  override val batchIncrement: Duration = Days(1)
}
