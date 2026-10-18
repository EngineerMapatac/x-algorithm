package com.twitter.simclusters_v2.scalding.embedding.tfg

import com.twitter.dal.client.dataset.KeyValDALDataset
import com.twitter.dal.client.dataset.SnapshotDALDatasetBase
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite.D
import com.twitter.scalding_internal.dalv2.DALWrite.WriteExtension
import com.twitter.scalding_internal.dalv2.remote_access.AllowCrossClusterSameDC
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.hdfs_sources.EntityEmbeddingsSources
import com.twitter.simclusters_v2.scalding.embedding.common.EmbeddingUtil
import com.twitter.simclusters_v2.thriftscala.EmbeddingType
import com.twitter.simclusters_v2.thriftscala.ModelVersion
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingId
import com.twitter.simclusters_v2.thriftscala.TfgTopicEmbeddings
import com.twitter.simclusters_v2.thriftscala.UserToInterestedInClusterScores
import com.twitter.simclusters_v2.thriftscala.{SimClustersEmbedding => ThriftSimClustersEmbedding}
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import java.util.TimeZone

object FavTfgTopicEmbeddingsAdhocApp extends TfgBasedTopicEmbeddingsBaseApp with AdhocExecutionApp {
  override val isAdhoc: Boolean = true
  override val embeddingType: EmbeddingType = EmbeddingType.FavTfgTopic
  override val embeddingSource: KeyValDALDataset[
    KeyVal[SimClustersEmbeddingId, ThriftSimClustersEmbedding]
  ] = EntityEmbeddingsSources.FavTfgTopicEmbeddingsDataset
  override val pathSuffix: String = "fav_tfg_topic_embedding"
  override val modelVersion: ModelVersion = ModelVersion.Model20m145kUpdated
  override val parquetDataSource: SnapshotDALDatasetBase[TfgTopicEmbeddings] =
    EntityEmbeddingsSources.FavTfgTopicEmbeddingsParquetDataset
  override def scoreExtractor: UserToInterestedInClusterScores => Double = scores =>
    scores.favScore.getOrElse(0.0)
}

object FavTfgTopicEmbeddingsScheduledApp
    extends TfgBasedTopicEmbeddingsBaseApp
    with ScheduledExecutionApp {
  override val isAdhoc: Boolean = false
  override val embeddingType: EmbeddingType = EmbeddingType.FavTfgTopic
  override val embeddingSource: KeyValDALDataset[
    KeyVal[SimClustersEmbeddingId, ThriftSimClustersEmbedding]
  ] = EntityEmbeddingsSources.FavTfgTopicEmbeddingsDataset
  override val pathSuffix: String = "fav_tfg_topic_embedding"
  override val modelVersion: ModelVersion = ModelVersion.Model20m145kUpdated
  override val parquetDataSource: SnapshotDALDatasetBase[TfgTopicEmbeddings] =
    EntityEmbeddingsSources.FavTfgTopicEmbeddingsParquetDataset
  override def scoreExtractor: UserToInterestedInClusterScores => Double = scores =>
    scores.favScore.getOrElse(0.0)

  override val firstTime: RichDate = RichDate("2020-05-25")
  override val batchIncrement: Duration = Days(1)
}

object FavTfgTopicEmbeddings2020AdhocApp
    extends TfgBasedTopicEmbeddingsBaseApp
    with AdhocExecutionApp {
  override val isAdhoc: Boolean = true
  override val embeddingType: EmbeddingType = EmbeddingType.FavTfgTopic
  override val embeddingSource: KeyValDALDataset[
    KeyVal[SimClustersEmbeddingId, ThriftSimClustersEmbedding]
  ] = EntityEmbeddingsSources.FavTfgTopicEmbeddings2020Dataset
  override val pathSuffix: String = "fav_tfg_topic_embedding"
  override val modelVersion: ModelVersion = ModelVersion.Model20m145k2020
  override val parquetDataSource: SnapshotDALDatasetBase[TfgTopicEmbeddings] =
    EntityEmbeddingsSources.FavTfgTopicEmbeddings2020ParquetDataset
  override def scoreExtractor: UserToInterestedInClusterScores => Double = scores =>
    scores.favScore.getOrElse(0.0)
}

object FavTfgTopicEmbeddings2020ScheduledApp
    extends TfgBasedTopicEmbeddingsBaseApp
    with ScheduledExecutionApp {
  override val isAdhoc: Boolean = false
  override val embeddingType: EmbeddingType = EmbeddingType.FavTfgTopic
  override val embeddingSource: KeyValDALDataset[
    KeyVal[SimClustersEmbeddingId, ThriftSimClustersEmbedding]
  ] = EntityEmbeddingsSources.FavTfgTopicEmbeddings2020Dataset
  override val pathSuffix: String = "fav_tfg_topic_embedding"
  override val modelVersion: ModelVersion = ModelVersion.Model20m145k2020
  override val parquetDataSource: SnapshotDALDatasetBase[TfgTopicEmbeddings] =
    EntityEmbeddingsSources.FavTfgTopicEmbeddings2020ParquetDataset
  override def scoreExtractor: UserToInterestedInClusterScores => Double = scores =>
    scores.favScore.getOrElse(0.0)

  override val firstTime: RichDate = RichDate("2021-03-10")
  override val batchIncrement: Duration = Days(1)
}

object FavTfgTopicEmbeddings2020CopyScheduledApp extends ScheduledExecutionApp {
  val isAdhoc: Boolean = false
  val embeddingType: EmbeddingType = EmbeddingType.FavTfgTopic
  val embeddingSource: KeyValDALDataset[
    KeyVal[SimClustersEmbeddingId, ThriftSimClustersEmbedding]
  ] = EntityEmbeddingsSources.FavTfgTopicEmbeddings2020Dataset
  val pathSuffix: String = "fav_tfg_topic_embedding"
  val modelVersion: ModelVersion = ModelVersion.Model20m145k2020

  override val firstTime: RichDate = RichDate("2023-01-20")
  override val batchIncrement: Duration = Days(3)

  def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    DAL
      .readMostRecentSnapshotNoOlderThan(
        EntityEmbeddingsSources.FavTfgTopicEmbeddings2020Dataset,
        Days(21))
      .withRemoteReadPolicy(AllowCrossClusterSameDC)
      .toTypedPipe
      .writeDALVersionedKeyValExecution(
        EntityEmbeddingsSources.FavTfgTopicEmbeddings2020Dataset,
        D.Suffix(
          EmbeddingUtil
            .getHdfsPath(isAdhoc = isAdhoc, isManhattanKeyVal = true, modelVersion, pathSuffix))
      )
  }
}
