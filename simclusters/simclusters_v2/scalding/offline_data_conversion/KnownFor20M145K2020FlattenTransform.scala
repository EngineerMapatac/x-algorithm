package com.twitter.simclusters_v2.scalding.offline_data_conversion

import com.twitter.scalding.typed.TypedPipe
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.common.ModelVersions
import com.twitter.simclusters_v2.thriftscala.ClustersUserIsKnownFor
import com.twitter.simclusters_v2.thriftscala.UserKnownForCluster
import com.twitter.util.Await

object KnownFor20M145K2020FlattenTransform {
  val asyncMeterBurstSizePerSecond: Int = 1
  val numReducers: Int = 100

  val asyncMeter: AsyncMeterLimiter =
    new AsyncMeterLimiter(asyncMeterBurstSizePerSecond)

  def flattenAndHydrateKnownForUserClusters(
    knownForSimclusters: TypedPipe[KeyVal[Long, ClustersUserIsKnownFor]]
  ): TypedPipe[UserKnownForCluster] = {
    val clusterIdToUsers = knownForSimclusters.collect {
      case KeyVal(
            userId,
            ClustersUserIsKnownFor(ModelVersions.Model20M145K2020, clusterIdToScores)) =>
        clusterIdToScores.map {
          case (clusterId, userToKnownForClusterScores) =>
            (
              clusterId,
              userId,
              userToKnownForClusterScores.knownForScore
            )
        }
    }.flatten
    val labeledClusterIdToUsers = clusterIdToUsers
      .groupBy(_._1).withReducers(numReducers)
      .mapGroup { (clusterId: Int, users: Iterator[(Int, Long, Option[Double])]) =>
        val clusterLabel: Option[String] = Await.result(
          asyncMeter
            .run(f = HydrationHelper.hydrateClusterIdToLabelName(clusterId))
            .map(_.v)
        )
        users.map { r =>
          UserKnownForCluster(
            userId = r._2,
            clusterId = r._1,
            clusterLabel = clusterLabel,
            knownForScore = r._3
          )
        }
      }.values
    labeledClusterIdToUsers
  }
}
