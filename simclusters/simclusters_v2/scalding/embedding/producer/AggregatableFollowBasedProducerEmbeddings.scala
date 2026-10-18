package com.twitter.simclusters_v2.scalding.embedding.producer

import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.scalding_internal.source.lzo_scrooge.FixedPathLzoScrooge
import com.twitter.simclusters_v2.hdfs_sources.AggregatableProducerSimclustersEmbeddingsByFollowScore2020ScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.AggregatableProducerSimclustersEmbeddingsByFollowScore2020ThriftScalaDataset
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

object AggregatableFollowBasedProducerEmbeddings2020ScheduledApp
    extends AggregatableFollowBasedProducerEmbeddingsBaseApp
    with ScheduledExecutionApp {

  override val modelVersion: ModelVersion = ModelVersion.Model20m145k2020
  private val outputPath: String =
    "/user/cassowary/manhattan_sequence_files/producer_simclusters_aggregatable_embeddings_by_follow_score_20m145k2020"

  private val outputPathThrift: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = false,
    isManhattanKeyVal = false,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_follow_score_thrift"
  )

  override def batchIncrement: Duration = Days(7)

  override def firstTime: RichDate = RichDate("2021-11-10")

  override def writeToManhattan(
    output: TypedPipe[KeyVal[SimClustersEmbeddingId, SimClustersEmbedding]]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    output
      .writeDALVersionedKeyValExecution(
        AggregatableProducerSimclustersEmbeddingsByFollowScore2020ScalaDataset,
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
        dataset = AggregatableProducerSimclustersEmbeddingsByFollowScore2020ThriftScalaDataset,
        updateStep = D.Daily,
        pathLayout = D.Suffix(outputPathThrift),
        fmt = D.Parquet,
        endDate = dateRange.end
      )
  }
}

object AggregatableFollowBasedProducerEmbeddings2020AdhocApp
    extends AggregatableFollowBasedProducerEmbeddingsBaseApp
    with AdhocExecutionApp {

  override val modelVersion: ModelVersion = ModelVersion.Model20m145k2020

  private val outputPath: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = true,
    isManhattanKeyVal = true,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_follow_score"
  )

  private val outputPathThrift: String = EmbeddingUtil.getHdfsPath(
    isAdhoc = true,
    isManhattanKeyVal = false,
    modelVersion = modelVersion,
    pathSuffix = "producer_simclusters_aggregatable_embeddings_by_follow_score_thrift"
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

trait AggregatableFollowBasedProducerEmbeddingsBaseApp
    extends AggregatableProducerEmbeddingsBaseApp {
  override val userToProducerScoringFn: NeighborWithWeights => Double =
    _.followScoreNormalizedByNeighborFollowersL2.getOrElse(0.0)
  override val userToClusterScoringFn: UserToInterestedInClusterScores => Double =
    _.followScoreClusterNormalizedOnly.getOrElse(0.0)
  override val embeddingType: EmbeddingType = EmbeddingType.AggregatableFollowBasedProducer
}
