package com.twitter.simclusters_v2.scalding.offline_data_conversion

import com.twitter.scalding.typed.TypedPipe
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.common.ModelVersions
import com.twitter.simclusters_v2.thriftscala.ClustersUserIsInterestedIn
import com.twitter.simclusters_v2.thriftscala.UserInterestedInCluster
import com.twitter.simclusters_v2.thriftscala.UserToInterestedInClusterScores
import com.twitter.util.Await

object InterestedIn20M145K2020FlattenTransform {
  val asyncMeterBurstSizePerSecond: Int = 1
  val numReducers: Int = 100

  val asyncMeter: AsyncMeterLimiter =
    new AsyncMeterLimiter(asyncMeterBurstSizePerSecond)

  def flattenAndHydrateInterestedInUserClusters(
    interestedInSimclusters: TypedPipe[KeyVal[Long, ClustersUserIsInterestedIn]]
  ): TypedPipe[UserInterestedInCluster] = {
    val clusterIdToUsers: TypedPipe[(Int, Long, Option[Double], Option[Double])] =
      interestedInSimclusters.collect {
        case KeyVal(
              userId,
              ClustersUserIsInterestedIn(ModelVersions.Model20M145K2020, clusterIdToScores)) =>
          clusterIdToScores.map {
            case (clusterId, userToInterestedInClusterScores: UserToInterestedInClusterScores) =>
              (
                clusterId,
                userId,
                userToInterestedInClusterScores.followScoreClusterNormalizedOnly,
                userToInterestedInClusterScores.favScoreClusterNormalizedOnly
              )
          }
      }.flatten

    val labeledClusterIdToUsers: TypedPipe[UserInterestedInCluster] =
      clusterIdToUsers
        .groupBy(_._1).withReducers(numReducers)
        .mapGroup {
          (clusterId: Int, users: Iterator[(Int, Long, Option[Double], Option[Double])]) =>
            val clusterLabel: Option[String] = Await.result(
              asyncMeter
                .run(f = HydrationHelper.hydrateClusterIdToLabelName(clusterId))
                .map(_.v)
            )
            users.map { r =>
              UserInterestedInCluster(
                userId = r._2,
                clusterId = r._1,
                clusterLabel = clusterLabel,
                followScoreClusterNormalizedOnly = r._3,
                favScoreClusterNormalizedOnly = r._4
              )
            }
        }.values
    labeledClusterIdToUsers
  }
}
