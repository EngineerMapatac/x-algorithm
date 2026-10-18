package com.twitter.simclusters_v2.scalding.embedding.producer

import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.scalding_internal.source.lzo_scrooge.FixedPathLzoScrooge
import com.twitter.simclusters_v2.hdfs_sources.AggregatableProducerSimclustersEmbeddingsByLogFavScoreScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.AggregatableProducerSimclustersEmbeddingsByLogFavScoreThriftScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.AggregatableProducerSimclustersEmbeddingsByLogFavScore2020ScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.AggregatableProducerSimclustersEmbeddingsByLogFavScore2020ThriftScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.AggregatableProducerSimclustersEmbeddingsByLogFavScoreRelaxedFavEngagementThreshold2020ScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.AggregatableProducerSimclustersEmbeddingsByLogFavScoreRelaxedFavEngagementThreshold2020ThriftScalaDataset
import com.twitter.simclusters_v2.scalding.embedding.common.EmbeddingUtil
import com.twitter.simclusters_v2.thriftscala.EmbeddingType
import com.twitter.simclusters_v2.thriftscala.ModelVersion
import com.twitter.simclusters_v2.thriftscala.NeighborWithWeights
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbedding
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingId
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingWithId
import com.twitter.simclusters_v2.thriftscala.UserToInterestedInClusterScores
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import java.util.TimeZone

object AggregatableLogFavBasedProducerEmbeddingsScheduledApp
    extends AggregatableLogFavBasedProducerEmbeddingsBaseApp
    with ScheduledExecutionApp {

  override val modelVersion: ModelVersion = ModelVersion.Model20m145kUpdated
  private val outputPath: String =
    "/user/cassowary/manhattan_sequence_files/producer_simclusters_aggregatable_embeddings_by_logfav_score"

  private val outputPathThrift: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = false,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_logfav_score_thrift"
  )

  override def batchIncrement: Duration = Days(7)

  override def firstTime: RichDate = RichDate("2020-04-05")

  override def writeToManhattan(
    output: TypedPipe[KeyVal[SimClustersEmbeddingId, SimClustersEmbedding]]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    output
      .writeDALVersionedKeyValExecution(
        AggregatableProducerSimclustersEmbeddingsByLogFavScoreScalaDataset,
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
        dataset = AggregatableProducerSimclustersEmbeddingsByLogFavScoreThriftScalaDataset,
        updateStep = D.Daily,
        pathLayout = D.Suffix(outputPathThrift),
        fmt = D.Parquet,
        endDate = dateRange.end
      )
  }
}

object AggregatableLogFavBasedProducerEmbeddings2020ScheduledApp
    extends AggregatableLogFavBasedProducerEmbeddingsBaseApp
    with ScheduledExecutionApp {

  override val modelVersion: ModelVersion = ModelVersion.Model20m145k2020
  private val outputPath: String =
    "/user/cassowary/manhattan_sequence_files/producer_simclusters_aggregatable_embeddings_by_logfav_score_20m145k2020"

  private val outputPathThrift: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = false,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_logfav_score_thrift"
  )

  override def batchIncrement: Duration = Days(7)

  override def firstTime: RichDate = RichDate("2021-03-05")

  override def writeToManhattan(
    output: TypedPipe[KeyVal[SimClustersEmbeddingId, SimClustersEmbedding]]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    output
      .writeDALVersionedKeyValExecution(
        AggregatableProducerSimclustersEmbeddingsByLogFavScore2020ScalaDataset,
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
        dataset = AggregatableProducerSimclustersEmbeddingsByLogFavScore2020ThriftScalaDataset,
        updateStep = D.Daily,
        pathLayout = D.Suffix(outputPathThrift),
        fmt = D.Parquet,
        endDate = dateRange.end
      )
  }
}

object AggregatableLogFavBasedProducerEmbeddingsRelaxedFavEngagementThreshold2020ScheduledApp
    extends AggregatableLogFavBasedProducerEmbeddingsBaseApp
    with ScheduledExecutionApp {

  override val modelVersion: ModelVersion = ModelVersion.Model20m145k2020

  override val embeddingType: EmbeddingType = EmbeddingType.RelaxedAggregatableLogFavBasedProducer

  override val minNumFavers = 15

  private val outputPath: String =
    "/user/cassowary/manhattan_sequence_files/producer_simclusters_aggregatable_embeddings_by_logfav_score_relaxed_fav_engagement_threshold_20m145k2020"

  private val outputPathThrift: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = false,
    modelVersion = modelVersion,
    pathSuffix =
      "producer_simclusters_aggregatable_embeddings_by_logfav_score_relaxed_fav_score_threshold_thrift"
  )

  override def batchIncrement: Duration = Days(7)

  override def firstTime: RichDate = RichDate("2021-07-26")

  override def writeToManhattan(
    output: TypedPipe[KeyVal[SimClustersEmbeddingId, SimClustersEmbedding]]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    output
      .writeDALVersionedKeyValExecution(
        AggregatableProducerSimclustersEmbeddingsByLogFavScoreRelaxedFavEngagementThreshold2020ScalaDataset,
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
        dataset =
          AggregatableProducerSimclustersEmbeddingsByLogFavScoreRelaxedFavEngagementThreshold2020ThriftScalaDataset,
        updateStep = D.Daily,
        pathLayout = D.Suffix(outputPathThrift),
        fmt = D.Parquet,
        endDate = dateRange.end
      )
  }
}

object AggregatableLogFavBasedProducerEmbeddingsAdhocApp
    extends AggregatableLogFavBasedProducerEmbeddingsBaseApp
    with AdhocExecutionApp {

  override val modelVersion: ModelVersion = ModelVersion.Model20m145kUpdated

  private val outputPath: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = true,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_log_fav_score"
  )

  private val outputPathThrift: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = false,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_log_fav_score_thrift"
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

object AggregatableLogFavBasedProducerEmbeddings2020AdhocApp
    extends AggregatableLogFavBasedProducerEmbeddingsBaseApp
    with AdhocExecutionApp {

  override val modelVersion: ModelVersion = ModelVersion.Model20m145k2020

  private val outputPath: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = true,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_log_fav_score"
  )

  private val outputPathThrift: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = false,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_log_fav_score_thrift"
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

trait AggregatableLogFavBasedProducerEmbeddingsBaseApp
    extends AggregatableProducerEmbeddingsBaseApp {
  override val userToProducerScoringFn: NeighborWithWeights => Double = _.logFavScore.getOrElse(0.0)
  override val userToClusterScoringFn: UserToInterestedInClusterScores => Double =
    _.logFavScore.getOrElse(0.0)
  override val embeddingType: EmbeddingType = EmbeddingType.AggregatableLogFavBasedProducer
}
