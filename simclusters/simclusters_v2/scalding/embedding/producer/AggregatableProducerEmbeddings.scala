package com.twitter.simclusters_v2.scalding.embedding.producer

import com.twitter.scalding._
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.scalding_internal.source.lzo_scrooge.FixedPathLzoScrooge
import com.twitter.simclusters_v2.hdfs_sources.DataSources
import com.twitter.simclusters_v2.hdfs_sources.InterestedInSources
import com.twitter.simclusters_v2.scalding.common.matrix.SparseMatrix
import com.twitter.simclusters_v2.scalding.common.matrix.SparseRowMatrix
import com.twitter.simclusters_v2.scalding.embedding.ProducerEmbeddingsFromInterestedIn
import com.twitter.simclusters_v2.scalding.embedding.common.EmbeddingUtil.ClusterId
import com.twitter.simclusters_v2.scalding.embedding.common.EmbeddingUtil.ProducerId
import com.twitter.simclusters_v2.scalding.embedding.common.EmbeddingUtil.UserId
import com.twitter.simclusters_v2.scalding.embedding.common.SimClustersEmbeddingBaseJob
import com.twitter.simclusters_v2.thriftscala.EmbeddingType
import com.twitter.simclusters_v2.thriftscala._
import java.util.TimeZone

trait AggregatableProducerEmbeddingsBaseApp extends SimClustersEmbeddingBaseJob[ProducerId] {

  val userToProducerScoringFn: NeighborWithWeights => Double
  val userToClusterScoringFn: UserToInterestedInClusterScores => Double
  val modelVersion: ModelVersion

  val minNumFavers: Int = ProducerEmbeddingsFromInterestedIn.minNumFaversForProducer

  override def numClustersPerNoun: Int = 60

  override def numNounsPerClusters: Int = 500

  override def thresholdForEmbeddingScores: Double = 0.01

  override def prepareNounToUserMatrix(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): SparseMatrix[ProducerId, UserId, Double] = {

    SparseMatrix(
      ProducerEmbeddingsFromInterestedIn
        .getFilteredUserUserNormalizedGraph(
          DataSources.userUserNormalizedGraphSource,
          DataSources.userNormsAndCounts,
          userToProducerScoringFn,
          _.faverCount.exists(
            _ > minNumFavers
          )
        )
        .map {
          case (userId, (producerId, score)) =>
            (producerId, userId, score)
        })
  }

  override def prepareUserToClusterMatrix(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): SparseRowMatrix[UserId, ClusterId, Double] = {
    SparseRowMatrix(
      ProducerEmbeddingsFromInterestedIn
        .getUserSimClustersMatrix(
          InterestedInSources
            .simClustersInterestedInSource(modelVersion, dateRange.embiggen(Days(5)), timeZone),
          userToClusterScoringFn,
          modelVersion
        )
        .mapValues(_.toMap),
      isSkinnyMatrix = true
    )
  }

  def convertEmbeddingToAggregatableEmbeddings(
    embeddings: TypedPipe[(ProducerId, Seq[(ClusterId, Double)])]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): TypedPipe[(ProducerId, Seq[(ClusterId, Double)])] = {
    embeddings.join(prepareNounToUserMatrix.rowL2Norms).map {
      case (producerId, (embeddingVec, norm)) =>
        producerId -> embeddingVec.map {
          case (id, score) => (id, score * norm)
        }
    }
  }

  override final def writeClusterToNounsIndex(
    output: TypedPipe[(ClusterId, Seq[(ProducerId, Double)])]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = { Execution.unit }

  def writeToManhattan(
    output: TypedPipe[KeyVal[SimClustersEmbeddingId, SimClustersEmbedding]]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit]

  def writeToThrift(
    output: TypedPipe[SimClustersEmbeddingWithId]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit]

  val embeddingType: EmbeddingType

  override final def writeNounToClustersIndex(
    output: TypedPipe[(ProducerId, Seq[(ClusterId, Double)])]
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    val convertedEmbeddings = convertEmbeddingToAggregatableEmbeddings(output)
      .map {
        case (producerId, topSimClustersWithScore) =>
          val id = SimClustersEmbeddingId(
            embeddingType = embeddingType,
            modelVersion = modelVersion,
            internalId = InternalId.UserId(producerId))

          val embeddings = SimClustersEmbedding(topSimClustersWithScore.map {
            case (clusterId, score) => SimClusterWithScore(clusterId, score)
          })

          SimClustersEmbeddingWithId(id, embeddings)
      }

    val keyValuePairs = convertedEmbeddings.map { simClustersEmbeddingWithId =>
      KeyVal(simClustersEmbeddingWithId.embeddingId, simClustersEmbeddingWithId.embedding)
    }
    val manhattanExecution = writeToManhattan(keyValuePairs)

    val thriftExecution = writeToThrift(convertedEmbeddings)

    Execution.zip(manhattanExecution, thriftExecution).unit
  }
}
