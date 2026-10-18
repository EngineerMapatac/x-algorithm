package com.twitter.simclusters_v2.scalding.inferred_entities

import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.typed.TypedPipe
import com.twitter.simclusters_v2.common.ClusterId
import com.twitter.simclusters_v2.common.ModelVersions
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.hdfs_sources.EntityEmbeddingsSources
import com.twitter.simclusters_v2.thriftscala.ClusterType
import com.twitter.simclusters_v2.thriftscala.EmbeddingType
import com.twitter.simclusters_v2.thriftscala.InferredEntity
import com.twitter.simclusters_v2.thriftscala.ModelVersion
import com.twitter.simclusters_v2.thriftscala.SemanticCoreEntityWithScore
import com.twitter.simclusters_v2.thriftscala.SimClustersInferredEntities
import com.twitter.simclusters_v2.thriftscala.SimClustersSource
import java.util.TimeZone

object InferredEntities {
  val MHRootPath: String =
    "/user/cassowary/manhattan_sequence_files/simclusters_v2_inferred_entities"

  val InterestedIn2020 =
    SimClustersSource(ClusterType.InterestedIn, ModelVersion.Model20m145k2020)

  val Dec11KnownFor = SimClustersSource(ClusterType.KnownFor, ModelVersion.Model20m145kDec11)

  val UpdatedKnownFor = SimClustersSource(ClusterType.KnownFor, ModelVersion.Model20m145kUpdated)

  val KnownFor2020 = SimClustersSource(ClusterType.KnownFor, ModelVersion.Model20m145k2020)

  val MinLegibleEntityScore = 0.6

  def getLegibleEntityEmbeddings(
    dateRange: DateRange,
    timeZone: TimeZone
  ): TypedPipe[(ClusterId, Seq[SemanticCoreEntityWithScore])] = {
    val entityEmbeddings = EntityEmbeddingsSources
      .getReverseIndexedSemanticCoreEntityEmbeddingsSource(
        EmbeddingType.FavBasedSematicCoreEntity,
        ModelVersions.Model20M145K2020,
        dateRange.embiggen(Days(35)(timeZone))
      )
    filterEntityEmbeddingsByScore(entityEmbeddings, MinLegibleEntityScore)
  }

  def filterEntityEmbeddingsByScore(
    entityEmbeddings: TypedPipe[(ClusterId, Seq[SemanticCoreEntityWithScore])],
    minEntityScore: Double
  ): TypedPipe[(ClusterId, Seq[SemanticCoreEntityWithScore])] = {
    entityEmbeddings.flatMap {
      case (clusterId, entities) =>
        val validEntities = entities.filter { entity => entity.score >= minEntityScore }
        if (validEntities.nonEmpty) {
          Some((clusterId, validEntities))
        } else {
          None
        }

    }
  }

  def combineResults(
    results: TypedPipe[(UserId, Seq[InferredEntity])]*
  ): TypedPipe[(UserId, SimClustersInferredEntities)] = {
    results
      .reduceLeft(_ ++ _)
      .sumByKey
      .map {
        case (userId, inferredEntities) =>
          (userId, SimClustersInferredEntities(inferredEntities))
      }
  }
}
