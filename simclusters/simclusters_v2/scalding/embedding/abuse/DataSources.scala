package com.twitter.simclusters_v2.scalding.embedding.abuse

import com.twitter.data.proto.Flock
import com.twitter.scalding.DateOps
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.RichDate
import com.twitter.scalding.UniqueID
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.simclusters_v2.hdfs_sources.InterestedInSources
import com.twitter.simclusters_v2.scalding.common.matrix.SparseMatrix
import com.twitter.simclusters_v2.scalding.embedding.common.EmbeddingUtil.ClusterId
import com.twitter.simclusters_v2.scalding.embedding.common.EmbeddingUtil.UserId
import com.twitter.simclusters_v2.scalding.embedding.common.ExternalDataSources
import graphstore.common.FlockBlocksJavaDataset
import java.util.TimeZone

object DataSources {

  private val ValidEdgeStateId = 0
  val NumBlocksP95 = 49

  def getUserInterestedInSparseMatrix(
    implicit dateRange: DateRange,
    timeZone: TimeZone
  ): SparseMatrix[UserId, ClusterId, Double] = {
    val simClusters = ExternalDataSources.simClustersInterestInSource

    val simClusterMatrixEntries = simClusters
      .flatMap { keyVal =>
        keyVal.value.clusterIdToScores.flatMap {
          case (clusterId, score) =>
            score.favScore.map { favScore =>
              (keyVal.key, clusterId, favScore)
            }
        }
      }

    SparseMatrix.apply[UserId, ClusterId, Double](simClusterMatrixEntries)
  }

  def getUserInterestedInTruncatedKMatrix(
    topK: Int
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): SparseMatrix[UserId, ClusterId, Double] = {
    SparseMatrix(
      InterestedInSources
        .simClustersInterestedInUpdatedSource(dateRange, timeZone)
        .flatMap {
          case (userId, clustersUserIsInterestedIn) =>
            val sortedAndTruncatedList = clustersUserIsInterestedIn.clusterIdToScores
              .mapValues(_.favScore.getOrElse(0.0)).filter(_._2 > 0.0).toList.sortBy(-_._2).take(
                topK)
            sortedAndTruncatedList.map {
              case (clusterId, score) =>
                (userId, clusterId, score)
            }
        }
    )
  }

  def getFlockBlocksSparseMatrix(
    maxNumBlocks: Int,
    rangeForData: DateRange
  )(
    implicit dateRange: DateRange
  ): SparseMatrix[UserId, UserId, Double] = {
    implicit val tz: java.util.TimeZone = DateOps.UTC
    val userGivingBlocks = SparseMatrix.apply[UserId, UserId, Double](
      DAL
        .readMostRecentSnapshotNoOlderThan(FlockBlocksJavaDataset, Days(30))
        .toTypedPipe
        .flatMap { data: Flock.Edge =>
          if (data.getStateId == ValidEdgeStateId &&
            rangeForData.contains(RichDate(data.getUpdatedAt * 1000L))) {
            Some((data.getSourceId, data.getDestinationId, 1.0))
          } else {
            None
          }
        })
    val usersWithLegitBlocks = userGivingBlocks.rowL1Norms.collect {
      case (userId, l1Norm) if l1Norm <= maxNumBlocks =>
        userId
    }
    userGivingBlocks.filterRows(usersWithLegitBlocks)
  }
}
