package com.twitter.simclusters_v2.common

import com.twitter.simclusters_v2.thriftscala.EmbeddingType
import com.twitter.simclusters_v2.thriftscala.InternalId
import com.twitter.simclusters_v2.thriftscala.MultiEmbeddingType
import com.twitter.simclusters_v2.thriftscala.TopicId
import com.twitter.simclusters_v2.thriftscala.TopicSubId
import com.twitter.simclusters_v2.thriftscala.{SimClustersEmbeddingId => ThriftEmbeddingId}
import com.twitter.simclusters_v2.thriftscala.{
  SimClustersMultiEmbeddingId => ThriftMultiEmbeddingId
}

object SimClustersMultiEmbeddingId {

  private val MultiEmbeddingTypeToEmbeddingType: Map[MultiEmbeddingType, EmbeddingType] =
    Map(
      MultiEmbeddingType.LogFavApeBasedMuseTopic -> EmbeddingType.LogFavApeBasedMuseTopic,
      MultiEmbeddingType.TwiceUserInterestedIn -> EmbeddingType.TwiceUserInterestedIn,
    )

  private val EmbeddingTypeToMultiEmbeddingType: Map[EmbeddingType, MultiEmbeddingType] =
    MultiEmbeddingTypeToEmbeddingType.map(_.swap)

  def toEmbeddingType(multiEmbeddingType: MultiEmbeddingType): EmbeddingType = {
    MultiEmbeddingTypeToEmbeddingType.getOrElse(
      multiEmbeddingType,
      throw new IllegalArgumentException(s"Invalid type: $multiEmbeddingType"))
  }

  def toMultiEmbeddingType(embeddingType: EmbeddingType): MultiEmbeddingType = {
    EmbeddingTypeToMultiEmbeddingType.getOrElse(
      embeddingType,
      throw new IllegalArgumentException(s"Invalid type: $embeddingType")
    )
  }

  def toEmbeddingId(
    simClustersMultiEmbeddingId: ThriftMultiEmbeddingId,
    subId: Int
  ): ThriftEmbeddingId = {
    val internalId = simClustersMultiEmbeddingId.internalId match {
      case InternalId.TopicId(topicId) =>
        InternalId.TopicSubId(
          TopicSubId(topicId.entityId, topicId.language, topicId.country, subId))
      case _ =>
        throw new IllegalArgumentException(
          s"Invalid simClusters InternalId ${simClustersMultiEmbeddingId.internalId}")
    }
    ThriftEmbeddingId(
      toEmbeddingType(simClustersMultiEmbeddingId.embeddingType),
      simClustersMultiEmbeddingId.modelVersion,
      internalId
    )
  }

  def toSubId(simClustersEmbeddingId: ThriftEmbeddingId): Int = {
    simClustersEmbeddingId.internalId match {
      case InternalId.TopicSubId(topicSubId) =>
        topicSubId.subId
      case _ =>
        throw new IllegalArgumentException(
          s"Invalid SimClustersEmbeddingId InternalId type, $simClustersEmbeddingId")
    }
  }

  def toMultiEmbeddingId(
    simClustersEmbeddingId: ThriftEmbeddingId
  ): ThriftMultiEmbeddingId = {
    simClustersEmbeddingId.internalId match {
      case InternalId.TopicSubId(topicSubId) =>
        ThriftMultiEmbeddingId(
          toMultiEmbeddingType(simClustersEmbeddingId.embeddingType),
          simClustersEmbeddingId.modelVersion,
          InternalId.TopicId(TopicId(topicSubId.entityId, topicSubId.language, topicSubId.country))
        )

      case _ =>
        throw new IllegalArgumentException(
          s"Invalid SimClustersEmbeddingId InternalId type, $simClustersEmbeddingId")
    }
  }

}
