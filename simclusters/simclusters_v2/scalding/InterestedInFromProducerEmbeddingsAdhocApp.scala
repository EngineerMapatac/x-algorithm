package com.twitter.simclusters_v2.scalding

import com.twitter.dal.client.dataset.KeyValDALDataset
import com.twitter.scalding.Execution
import com.twitter.scalding.TypedTsv
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.common.ModelVersions
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.hdfs_sources.ProducerEmbeddingSources
import com.twitter.simclusters_v2.hdfs_sources.AdhocKeyValSources
import com.twitter.simclusters_v2.hdfs_sources.DataSources
import com.twitter.simclusters_v2.hdfs_sources.SimclustersV2InterestedInFromProducerEmbeddings20M145KUpdatedScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.UserAndNeighborsFixedPathSource
import com.twitter.simclusters_v2.hdfs_sources.UserUserNormalizedGraphScalaDataset
import com.twitter.simclusters_v2.scalding.common.Util
import com.twitter.simclusters_v2.thriftscala.ClustersUserIsInterestedIn
import com.twitter.simclusters_v2.thriftscala.EmbeddingType
import com.twitter.simclusters_v2.thriftscala.SimClusterWithScore
import com.twitter.simclusters_v2.thriftscala.TopSimClustersWithScore
import com.twitter.simclusters_v2.thriftscala.UserToInterestedInClusterScores
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import java.util.TimeZone
import scala.util.Random

object InterestedInFromProducerEmbeddingsAdhocApp extends AdhocExecutionApp {
  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {

    val outputDir = args("outputDir")
    val inputGraph = args.optional("graphInputDir") match {
      case Some(inputDir) => TypedPipe.from(UserAndNeighborsFixedPathSource(inputDir))
      case None =>
        DAL
          .readMostRecentSnapshotNoOlderThan(UserUserNormalizedGraphScalaDataset, Days(30))
          .toTypedPipe
    }
    val socialProofThreshold = args.int("socialProofThreshold", 2)
    val maxClustersPerUserFinalResult = args.int("maxInterestedInClustersPerUser", 50)
    val maxClustersFromProducer = args.int("maxClustersPerProducer", 25)
    val typedTsvTag = args.boolean("typedTsv")

    val embeddingType =
      EmbeddingType.ProducerFavBasedSemanticCoreEntity
    val modelVersion = ModelVersions.Model20M145KUpdated
    val producerEmbeddings = ProducerEmbeddingSources
      .producerEmbeddingSourceLegacy(embeddingType, ModelVersions.toModelVersion(modelVersion))(
        dateRange.embiggen(Days(7)))

    import InterestedInFromProducerEmbeddingsBatchApp._

    val numProducerMappings = Stat("num_producer_embeddings_total")
    val numProducersWithLargeClusterMappings = Stat(
      "num_producers_with_more_clusters_than_threshold")
    val numProducersWithSmallClusterMappings = Stat(
      "num_producers_with_clusters_less_than_threshold")
    val totalClustersCoverageProducerEmbeddings = Stat("num_clusters_total_producer_embeddings")

    val producerEmbeddingsWithScore = producerEmbeddings.map {
      case (userId: Long, topSimClusters: TopSimClustersWithScore) =>
        (
          userId,
          topSimClusters.topClusters.toArray
            .map {
              case (simCluster: SimClusterWithScore) =>
                (simCluster.clusterId, simCluster.score.toFloat)
            }
        )
    }
    val producerEmbeddingsPruned = producerEmbeddingsWithScore.map {
      case (producerId, clusterArray) =>
        numProducerMappings.inc()
        val clusterSize = clusterArray.size
        totalClustersCoverageProducerEmbeddings.incBy(clusterSize)
        val prunedList = if (clusterSize > maxClustersFromProducer) {
          numProducersWithLargeClusterMappings.inc()
          clusterArray
            .sortBy {
              case (_, knownForScore) => -knownForScore
            }.take(maxClustersFromProducer)
        } else {
          numProducersWithSmallClusterMappings.inc()
          clusterArray
        }
        (producerId, prunedList)
    }

    val result = InterestedInFromKnownFor
      .run(
        inputGraph,
        producerEmbeddingsPruned,
        socialProofThreshold,
        maxClustersPerUserFinalResult,
        modelVersion
      )

    val resultWithoutSocial = getInterestedInDiscardSocial(result)

    if (typedTsvTag) {
      Util.printCounters(
        resultWithoutSocial
          .map {
            case (userId: Long, clusters: ClustersUserIsInterestedIn) =>
              (
                userId,
                clusters.clusterIdToScores.keys.toString()
              )
          }
          .writeExecution(
            TypedTsv(outputDir)
          )
      )
    } else {
      Util.printCounters(
        resultWithoutSocial
          .writeExecution(
            AdhocKeyValSources.interestedInSource(outputDir)
          )
      )
    }
  }
}

object InterestedInFromProducerEmbeddingsBatchApp extends ScheduledExecutionApp {
  override val firstTime: RichDate = RichDate("2019-11-01")

  override val batchIncrement: Duration = Days(7)

  def getPrunedEmbeddings(
    producerEmbeddings: TypedPipe[(Long, TopSimClustersWithScore)],
    maxClustersFromProducer: Int
  ): TypedPipe[(Long, TopSimClustersWithScore)] = {
    producerEmbeddings.map {
      case (producerId, producerClusters) =>
        val prunedProducerClusters =
          producerClusters.topClusters
            .sortBy {
              case simCluster => -simCluster.score.toFloat
            }.take(maxClustersFromProducer)
        (producerId, TopSimClustersWithScore(prunedProducerClusters, producerClusters.modelVersion))
    }
  }

  def getInterestedInDiscardSocial(
    interestedInFromProducersResult: TypedPipe[(UserId, ClustersUserIsInterestedIn)]
  ): TypedPipe[(UserId, ClustersUserIsInterestedIn)] = {
    interestedInFromProducersResult.map {
      case (srcId, fullClusterList) =>
        val fullClusterListWithoutSocial = fullClusterList.clusterIdToScores.map {
          case (clusterId, clusterDetails) =>
            val clusterDetailsWithoutSocial = UserToInterestedInClusterScores(
              followScore = clusterDetails.followScore,
              followScoreClusterNormalizedOnly = clusterDetails.followScoreClusterNormalizedOnly,
              followScoreProducerNormalizedOnly = clusterDetails.followScoreProducerNormalizedOnly,
              followScoreClusterAndProducerNormalized =
                clusterDetails.followScoreClusterAndProducerNormalized,
              favScore = clusterDetails.favScore,
              favScoreClusterNormalizedOnly = clusterDetails.favScoreClusterNormalizedOnly,
              favScoreProducerNormalizedOnly = clusterDetails.favScoreProducerNormalizedOnly,
              favScoreClusterAndProducerNormalized =
                clusterDetails.favScoreClusterAndProducerNormalized,
              usersBeingFollowed = None,
              usersThatWereFaved = None,
              numUsersInterestedInThisClusterUpperBound =
                clusterDetails.numUsersInterestedInThisClusterUpperBound,
              logFavScore = clusterDetails.logFavScore,
              logFavScoreClusterNormalizedOnly = clusterDetails.logFavScoreClusterNormalizedOnly,
              numUsersBeingFollowed = Some(clusterDetails.usersBeingFollowed.getOrElse(Nil).size),
              numUsersThatWereFaved = Some(clusterDetails.usersThatWereFaved.getOrElse(Nil).size)
            )
            (clusterId, clusterDetailsWithoutSocial)
        }
        (
          srcId,
          ClustersUserIsInterestedIn(
            fullClusterList.knownForModelVersion,
            fullClusterListWithoutSocial))
    }
  }

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {

    val socialProofThreshold = args.int("socialProofThreshold", 2)
    val maxClustersFromProducer = args.int("maxClustersPerProducer", 25)
    val maxClustersPerUserFinalResult = args.int("maxInterestedInClustersPerUser", 50)

    val modelVersionUpdated = ModelVersions.toModelVersion(ModelVersions.Model20M145KUpdated)
    val rootPath: String = s"/user/cassowary/manhattan_sequence_files"
    val interestedInFromProducersPath =
      rootPath + "/interested_in_from_producer_embeddings/" + modelVersionUpdated

    val userUserNormalGraph =
      DataSources.userUserNormalizedGraphSource(dateRange.prepend(Days(7))).forceToDisk
    val outputKVDataset: KeyValDALDataset[KeyVal[Long, ClustersUserIsInterestedIn]] =
      SimclustersV2InterestedInFromProducerEmbeddings20M145KUpdatedScalaDataset
    val producerEmbeddings = ProducerEmbeddingSources
      .producerEmbeddingSourceLegacy(
        EmbeddingType.ProducerFavBasedSemanticCoreEntity,
        modelVersionUpdated)(dateRange.embiggen(Days(7)))

    val producerEmbeddingsPruned = getPrunedEmbeddings(producerEmbeddings, maxClustersFromProducer)
    val producerEmbeddingsWithScore = producerEmbeddingsPruned.map {
      case (userId: Long, topSimClusters: TopSimClustersWithScore) =>
        (
          userId,
          topSimClusters.topClusters.toArray
            .map {
              case (simCluster: SimClusterWithScore) =>
                (simCluster.clusterId, simCluster.score.toFloat)
            }
        )
    }

    val interestedInFromProducersResult =
      InterestedInFromKnownFor.run(
        userUserNormalGraph,
        producerEmbeddingsWithScore,
        socialProofThreshold,
        maxClustersPerUserFinalResult,
        modelVersionUpdated.toString
      )

    val interestedInFromProducersWithoutSocial =
      getInterestedInDiscardSocial(interestedInFromProducersResult)

    val writeKeyValResultExec = interestedInFromProducersWithoutSocial
      .map { case (userId, clusters) => KeyVal(userId, clusters) }
      .writeDALVersionedKeyValExecution(
        outputKVDataset,
        D.Suffix(interestedInFromProducersPath)
      )
    writeKeyValResultExec
  }

}
