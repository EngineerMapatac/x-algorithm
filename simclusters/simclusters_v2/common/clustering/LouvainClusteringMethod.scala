package com.twitter.simclusters_v2.common.clustering

import com.twitter.eventdetection.common.louvain.LouvainDriver
import com.twitter.eventdetection.common.louvain.NetworkFactory
import com.twitter.eventdetection.common.model.Entity
import com.twitter.eventdetection.common.model.NetworkInput
import com.twitter.eventdetection.common.model.TextEntityValue
import com.twitter.util.Stopwatch
import scala.collection.JavaConverters._
import scala.math.max

class LouvainClusteringMethod(
  similarityThreshold: Double,
  appliedResolutionFactor: Option[Double])
    extends ClusteringMethod {

  import ClusteringStatistics._

  def cluster[T](
    embeddings: Map[Long, T],
    similarityFn: (T, T) => Double,
    recordStatCallback: (String, Long) => Unit = (_, _) => ()
  ): Set[Set[Long]] = {

    val timeSinceGraphBuildStart = Stopwatch.start()
    val edges: Seq[((Long, Long), Double)] = embeddings.toSeq
      .combinations(2)
      .map { pair: Seq[(Long, T)] =>
        val (user1, embedding1) = pair.head
        val (user2, embedding2) = pair(1)
        val similarity = similarityFn(embedding1, embedding2)

        recordStatCallback(
          StatComputedSimilarityBeforeFilter,
          (similarity * 100).toLong
        )

        ((user1, user2), similarity)
      }
      .filter(_._2 > similarityThreshold)
      .toSeq

    recordStatCallback(StatSimilarityGraphTotalBuildTime, timeSinceGraphBuildStart().inMilliseconds)

    val individualClusters: Set[Long] = embeddings.keySet -- edges.flatMap {
      case ((user1, user2), _) => Set(user1, user2)
    }.toSet

    val embeddingIdToEntity: Map[Long, Entity] = embeddings.map {
      case (id, _) => id -> Entity(TextEntityValue(id.toString, Some(id.toString)), None)
    }
    val entityToEmbeddingId: Map[Entity, Long] = embeddingIdToEntity.map {
      case (id, e) => e -> id
    }

    val networkInputList = edges
      .map {
        case ((fromUserId: Long, toUserId: Long), weight: Double) =>
          new NetworkInput(embeddingIdToEntity(fromUserId), embeddingIdToEntity(toUserId), weight)
      }.toList.asJava

    val timeSinceClusteringAlgRunStart = Stopwatch.start()
    val networkDictionary = NetworkFactory.buildDictionary(networkInputList)
    val network = NetworkFactory.buildNetwork(networkInputList, networkDictionary)

    if (networkInputList.size() == 0) {
      embeddings.keySet.map(e => Set(e))
    } else {
      val clusteredIds = appliedResolutionFactor match {
        case Some(res) =>
          LouvainDriver.clusterAppliedResolutionFactor(network, networkDictionary, res)
        case None => LouvainDriver.cluster(network, networkDictionary)
      }

      recordStatCallback(
        StatClusteringAlgorithmRunTime,
        timeSinceClusteringAlgRunStart().inMilliseconds)

      val atLeast2MembersClusters: Set[Set[Long]] = clusteredIds.asScala
        .groupBy(_._2)
        .mapValues(_.map { case (e, _) => entityToEmbeddingId(e) }.toSet)
        .values.toSet

      atLeast2MembersClusters ++ individualClusters.map { e => Set(e) }

    }
  }

  def clusterWithSilhouette[T](
    embeddings: Map[Long, T],
    similarityFn: (T, T) => Double,
    similarityFnForSil: (T, T) => Double,
    recordStatCallback: (String, Long) => Unit = (_, _) => ()
  ): (Set[Set[Long]], Set[Set[(Long, Double)]]) = {

    val timeSinceGraphBuildStart = Stopwatch.start()
    val edgesSimilarityMap = collection.mutable.Map[(Long, Long), Double]()

    val edges: Seq[((Long, Long), Double)] = embeddings.toSeq
      .combinations(2)
      .map { pair: Seq[(Long, T)] =>
        val (user1, embedding1) = pair.head
        val (user2, embedding2) = pair(1)
        val similarity = similarityFn(embedding1, embedding2)
        val similarityForSil = similarityFnForSil(embedding1, embedding2)
        edgesSimilarityMap.put((user1, user2), similarityForSil)
        edgesSimilarityMap.put((user2, user1), similarityForSil)

        recordStatCallback(
          StatComputedSimilarityBeforeFilter,
          (similarity * 100).toLong
        )

        ((user1, user2), similarity)
      }
      .filter(_._2 > similarityThreshold)
      .toSeq

    recordStatCallback(StatSimilarityGraphTotalBuildTime, timeSinceGraphBuildStart().inMilliseconds)

    val individualClusters: Set[Long] = embeddings.keySet -- edges.flatMap {
      case ((user1, user2), _) => Set(user1, user2)
    }.toSet

    val embeddingIdToEntity: Map[Long, Entity] = embeddings.map {
      case (id, _) => id -> Entity(TextEntityValue(id.toString, Some(id.toString)), None)
    }
    val entityToEmbeddingId: Map[Entity, Long] = embeddingIdToEntity.map {
      case (id, e) => e -> id
    }

    val networkInputList = edges
      .map {
        case ((fromUserId: Long, toUserId: Long), weight: Double) =>
          new NetworkInput(embeddingIdToEntity(fromUserId), embeddingIdToEntity(toUserId), weight)
      }.toList.asJava

    val timeSinceClusteringAlgRunStart = Stopwatch.start()
    val networkDictionary = NetworkFactory.buildDictionary(networkInputList)
    val network = NetworkFactory.buildNetwork(networkInputList, networkDictionary)

    val clusters = if (networkInputList.size() == 0) {
      embeddings.keySet.map(e => Set(e))
    } else {
      val clusteredIds = appliedResolutionFactor match {
        case Some(res) =>
          LouvainDriver.clusterAppliedResolutionFactor(network, networkDictionary, res)
        case None => LouvainDriver.cluster(network, networkDictionary)
      }

      recordStatCallback(
        StatClusteringAlgorithmRunTime,
        timeSinceClusteringAlgRunStart().inMilliseconds)

      val atLeast2MembersClusters: Set[Set[Long]] = clusteredIds.asScala
        .groupBy(_._2)
        .mapValues(_.map { case (e, _) => entityToEmbeddingId(e) }.toSet)
        .values.toSet

      atLeast2MembersClusters ++ individualClusters.map { e => Set(e) }

    }

    val contactIdWithSilhouette = clusters.map {
      case cluster =>
        val otherClusters = clusters - cluster

        cluster.map {
          case contactId =>
            if (otherClusters.isEmpty) {
              (contactId, 0.0)
            } else {
              val otherSameClusterContacts = cluster - contactId

              if (otherSameClusterContacts.isEmpty) {
                (contactId, 0.0)
              } else {
                val a_i = otherSameClusterContacts.map {
                  case sameClusterContact =>
                    edgesSimilarityMap((contactId, sameClusterContact))
                }.sum / otherSameClusterContacts.size

                val b_i = otherClusters.map {
                  case otherCluster =>
                    otherCluster.map {
                      case otherClusterContact =>
                        edgesSimilarityMap((contactId, otherClusterContact))
                    }.sum / otherCluster.size
                }.max

                val s_i = (a_i - b_i) / max(a_i, b_i)
                (contactId, s_i)
              }
            }
        }
    }

    (clusters, contactIdWithSilhouette)
  }
}
