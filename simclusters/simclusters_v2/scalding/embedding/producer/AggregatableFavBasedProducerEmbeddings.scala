package com.twitter.simclusters_v2.scalding.embedding.producer

import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.scalding_internal.source.lzo_scrooge.FixedPathLzoScrooge
import com.twitter.simclusters_v2.hdfs_sources.AggregatableProducerSimclustersEmbeddingsByFavScoreScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.AggregatableProducerSimclustersEmbeddingsByFavScoreThriftScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.AggregatableProducerSimclustersEmbeddingsByFavScore2020ScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.AggregatableProducerSimclustersEmbeddingsByFavScore2020ThriftScalaDataset
import com.twitter.simclusters_v2.scalding.embedding.common.EmbeddingUtil
import com.twitter.simclusters_v2.thriftscala._
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import java.util.TimeZone

object AggregatableFavBasedProducerEmbeddingsScheduledApp
    extends AggregatableFavBasedProducerEmbeddingsBaseApp
    with ScheduledExecutionApp {

  override val modelVersion: ModelVersion = ModelVersion.Model20m145kUpdated
  private val outputPath: String =
    "/user/cassowary/manhattan_sequence_files/producer_simclusters_aggregatable_embeddings_by_fav_score"

  private val outputPathThrift: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = false,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_fav_score_thrift"
  )

  override def firstTime: RichDate = RichDate("2020-05-11")

  override def batchIncrement: Duration = Days(7)

  override def writeToManhattan(
    output: TypedPipe[KeyVal[SimClustersEmbeddingId, SimClustersEmbedding]]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    output
      .writeDALVersionedKeyValExecution(
        AggregatableProducerSimclustersEmbeddingsByFavScoreScalaDataset,
        D.Suffix(outputPath),
        version = ExplicitEndTime(dateRange.end)
      )
  }

  override def writeToThrift(
    output: TypedPipe[SimClustersEmbeddingWithId]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    output
      .writeDALSnapshotExecution(
        dataset = AggregatableProducerSimclustersEmbeddingsByFavScoreThriftScalaDataset,
        updateStep = D.Daily,
        pathLayout = D.Suffix(outputPathThrift),
        fmt = D.Parquet,
        endDate = dateRange.end
      )
  }
}

object AggregatableFavBasedProducerEmbeddings2020ScheduledApp
    extends AggregatableFavBasedProducerEmbeddingsBaseApp
    with ScheduledExecutionApp {

  override val modelVersion: ModelVersion = ModelVersion.Model20m145k2020
  private val outputPath: String =
    "/user/cassowary/manhattan_sequence_files/producer_simclusters_aggregatable_embeddings_by_fav_score_20m145k2020"

  private val outputPathThrift: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = false,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_fav_score_thrift"
  )

  override def firstTime: RichDate = RichDate("2021-03-04")

  override def batchIncrement: Duration = Days(7)

  override def writeToManhattan(
    output: TypedPipe[KeyVal[SimClustersEmbeddingId, SimClustersEmbedding]]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    output
      .writeDALVersionedKeyValExecution(
        AggregatableProducerSimclustersEmbeddingsByFavScore2020ScalaDataset,
        D.Suffix(outputPath),
        version = ExplicitEndTime(dateRange.end)
      )
  }

  override def writeToThrift(
    output: TypedPipe[SimClustersEmbeddingWithId]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    output
      .writeDALSnapshotExecution(
        dataset = AggregatableProducerSimclustersEmbeddingsByFavScore2020ThriftScalaDataset,
        updateStep = D.Daily,
        pathLayout = D.Suffix(outputPathThrift),
        fmt = D.Parquet,
        endDate = dateRange.end
      )
  }
}

object AggregatableFavBasedProducerEmbeddingsAdhocApp
    extends AggregatableFavBasedProducerEmbeddingsBaseApp
    with AdhocExecutionApp {

  override val modelVersion: ModelVersion = ModelVersion.Model20m145kUpdated
  private val outputPath: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = true,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_fav_score"
  )

  private val outputPathThrift: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = false,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_fav_score_thrift"
  )

  override def writeToManhattan(
    output: TypedPipe[KeyVal[SimClustersEmbeddingId, SimClustersEmbedding]]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    output
      .flatMap { keyVal =>
        keyVal.value.embedding.map { simClusterWithScore =>
          (
            keyVal.key.embeddingType,
            keyVal.key.modelVersion,
            keyVal.key.internalId,
            simClusterWithScore.clusterId,
            simClusterWithScore.score
          )
        }
      }
      .writeExecution(
        TypedTsv(outputPath)
      )
  }

  override def writeToThrift(
    output: TypedPipe[SimClustersEmbeddingWithId]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    output
      .writeExecution(
        new FixedPathLzoScrooge(outputPathThrift, SimClustersEmbeddingWithId)
      )
  }
}

object AggregatableFavBasedProducerEmbeddings2020AdhocApp
    extends AggregatableFavBasedProducerEmbeddingsBaseApp
    with AdhocExecutionApp {

  override val modelVersion: ModelVersion = ModelVersion.Model20m145k2020
  private val outputPath: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = true,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_fav_score"
  )

  private val outputPathThrift: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = false,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_fav_score_thrift"
  )

  override def writeToManhattan(
    output: TypedPipe[KeyVal[SimClustersEmbeddingId, SimClustersEmbedding]]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    output
      .flatMap { keyVal =>
        keyVal.value.embedding.map { simClusterWithScore =>
          (
            keyVal.key.embeddingType,
            keyVal.key.modelVersion,
            keyVal.key.internalId,
            simClusterWithScore.clusterId,
            simClusterWithScore.score
          )
        }
      }
      .writeExecution(
        TypedTsv(outputPath)
      )
  }

  override def writeToThrift(
    output: TypedPipe[SimClustersEmbeddingWithId]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    output
      .writeExecution(
        new FixedPathLzoScrooge(outputPathThrift, SimClustersEmbeddingWithId)
      )
  }
}

trait AggregatableFavBasedProducerEmbeddingsBaseApp extends AggregatableProducerEmbeddingsBaseApp {
  override val userToProducerScoringFn: NeighborWithWeights => Double =
    _.favScoreHalfLife100Days.getOrElse(0.0)
  override val userToClusterScoringFn: UserToInterestedInClusterScores => Double =
    _.favScore.getOrElse(0.0)
  override val embeddingType: EmbeddingType = EmbeddingType.AggregatableFavBasedProducer
}
