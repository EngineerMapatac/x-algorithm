package com.twitter.simclusters_v2.scalding.inferred_entities

import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.common.ModelVersions
import com.twitter.simclusters_v2.common.SemanticCoreEntityId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.hdfs_sources.SimclustersInferredEntitiesFromKnownForScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.SimclustersV2InterestedIn20M145KUpdatedScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.SimclustersV2InterestedInScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.SimclustersV2KnownFor20M145KDec11ScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.SimclustersV2KnownFor20M145KUpdatedScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.UserUserNormalizedGraphScalaDataset
import com.twitter.simclusters_v2.scalding.KnownForSources
import com.twitter.simclusters_v2.thriftscala.EntitySource
import com.twitter.simclusters_v2.thriftscala.SimClusterWithScore
import com.twitter.simclusters_v2.thriftscala.SimClustersSource
import com.twitter.simclusters_v2.thriftscala.TopSimClustersWithScore
import com.twitter.simclusters_v2.thriftscala.UserAndNeighbors
import java.util.TimeZone

object ProdSources {

  def getDec11KnownFor(implicit tz: TimeZone): TypedPipe[(UserId, Seq[SimClusterWithScore])] =
    KnownForSources
      .readDALDataset(
        SimclustersV2KnownFor20M145KDec11ScalaDataset,
        Days(30),
        ModelVersions.Model20M145KDec11)
      .map {
        case (userId, clustersArray) =>
          val clusters = clustersArray.map {
            case (clusterId, score) => SimClusterWithScore(clusterId, score)
          }.toSeq
          (userId, clusters)
      }

  def getUpdatedKnownFor(implicit tz: TimeZone): TypedPipe[(UserId, Seq[SimClusterWithScore])] =
    KnownForSources
      .readDALDataset(
        SimclustersV2KnownFor20M145KUpdatedScalaDataset,
        Days(30),
        ModelVersions.Model20M145KUpdated
      )
      .map {
        case (userId, clustersArray) =>
          val clusters = clustersArray.map {
            case (clusterId, score) => SimClusterWithScore(clusterId, score)
          }.toSeq
          (userId, clusters)
      }

  def getInferredEntitiesFromKnownFor(
    inferredFromCluster: SimClustersSource,
    inferredFromEntity: EntitySource,
    dateRange: DateRange
  ): TypedPipe[(UserId, Seq[(SemanticCoreEntityId, Double)])] = {
    DAL
      .readMostRecentSnapshot(SimclustersInferredEntitiesFromKnownForScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .map {
        case KeyVal(userId, entities) =>
          val validEntities =
            entities.entities
              .collect {
                case entity
                    if entity.entitySource.contains(inferredFromEntity) &&
                      entity.simclusterSource.contains(inferredFromCluster) =>
                  (entity.entityId, entity.score)
              }
              .groupBy(_._1)
              .map { case (entityId, scores) => (entityId, scores.map(_._2).max) }
              .toSeq
          (userId, validEntities)
      }
  }

  def getUserUserEngagementGraph(dateRange: DateRange): TypedPipe[UserAndNeighbors] = {
    DAL
      .readMostRecentSnapshot(UserUserNormalizedGraphScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
  }
}
