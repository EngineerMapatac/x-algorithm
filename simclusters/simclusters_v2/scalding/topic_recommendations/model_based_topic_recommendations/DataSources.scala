package com.twitter.simclusters_v2.scalding.topic_recommendations.model_based_topic_recommendations

import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.Stat
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.UniqueID
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.Proc3Atla
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.common.Language
import com.twitter.simclusters_v2.common.TopicId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.hdfs_sources.FavTfgTopicEmbeddingsScalaDataset
import com.twitter.simclusters_v2.scalding.embedding.common.ExternalDataSources
import com.twitter.simclusters_v2.summingbird.stores.UserInterestedInReadableStore
import com.twitter.simclusters_v2.thriftscala.EmbeddingType
import com.twitter.simclusters_v2.thriftscala.InternalId
import com.twitter.simclusters_v2.thriftscala.LocaleEntityId
import com.twitter.simclusters_v2.thriftscala.ModelVersion
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingId
import java.util.TimeZone

object DataSources {

  private val topicEmbeddingDataset = FavTfgTopicEmbeddingsScalaDataset
  private val topicEmbeddingType = EmbeddingType.FavTfgTopic

  def getUserInterestedInData(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): TypedPipe[(UserId, Map[Int, Double])] = {
    val numUserInterestedInInput = Stat("num_user_interested_in")
    ExternalDataSources.simClustersInterestInSource
      .map {
        case KeyVal(userId, clustersUserIsInterestedIn) =>
          val clustersPostFiltering = clustersUserIsInterestedIn.clusterIdToScores.filter {
            case (clusterId, clusterScores) =>
              clusterScores.numUsersInterestedInThisClusterUpperBound.exists(
                _ < UserInterestedInReadableStore.MaxClusterSizeForUserInterestedInDataset)
          }
          numUserInterestedInInput.inc()
          (userId, clustersPostFiltering.mapValues(_.favScore.getOrElse(0.0)).toMap)
      }
  }

  def getPerLanguageTopicEmbeddings(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): TypedPipe[((TopicId, Language), Map[Int, Double])] = {
    val numTFGPerLanguageEmbeddings = Stat("num_per_language_tfg_embeddings")
    DAL
      .readMostRecentSnapshotNoOlderThan(topicEmbeddingDataset, Days(30))
      .withRemoteReadPolicy(ExplicitLocation(Proc3Atla))
      .toTypedPipe
      .map {
        case KeyVal(k, v) => (k, v)
      }.collect {
        case (
              SimClustersEmbeddingId(
                embedType,
                ModelVersion.Model20m145kUpdated,
                InternalId.LocaleEntityId(LocaleEntityId(entityId, lang))),
              embedding) if (embedType == topicEmbeddingType) =>
          numTFGPerLanguageEmbeddings.inc()
          ((entityId, lang), embedding.embedding.map(_.toTuple).toMap)
      }.forceToDisk
  }
}
