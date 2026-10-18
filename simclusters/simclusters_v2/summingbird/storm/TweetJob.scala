package com.twitter.simclusters_v2.summingbird.storm

import com.twitter.simclusters_v2.common.ModelVersions._
import com.twitter.simclusters_v2.summingbird.common.SimClustersProfile.SimClustersTweetProfile
import com.twitter.simclusters_v2.summingbird.common.Configs
import com.twitter.simclusters_v2.summingbird.common.Implicits
import com.twitter.simclusters_v2.summingbird.common.SimClustersHashUtil
import com.twitter.simclusters_v2.summingbird.common.SimClustersInterestedInUtil
import com.twitter.simclusters_v2.summingbird.common.StatsUtil
import com.twitter.simclusters_v2.thriftscala._
import com.twitter.snowflake.id.SnowflakeId
import com.twitter.summingbird._
import com.twitter.summingbird.option.JobId
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.summingbird.stores.TweetMetadata
import com.twitter.unified_user_actions.thriftscala.ActionType
import com.twitter.unified_user_actions.thriftscala.Item.TweetInfo
import com.twitter.unified_user_actions.thriftscala.UnifiedUserAction
import com.twitter.unified_user_actions.thriftscala.UserIdentifier

object TweetJob {

  import Implicits._
  import StatsUtil._

  object NodeName {
    final val TweetClusterScoreFlatMapNodeName: String = "TweetClusterScoreFlatMap"
    final val TweetClusterUpdatedScoresFlatMapNodeName: String = "TweetClusterUpdatedScoreFlatMap"
    final val TweetClusterScoreSummerNodeName: String = "TweetClusterScoreSummer"
    final val TweetTopKNodeName: String = "TweetTopKSummer"
    final val ClusterTopKTweetsNodeName: String = "ClusterTopKTweetsSummer"
    final val ClusterTopKHydratedTweetsNodeName: String = "ClusterTopKTweetsWithAuthorSummer"
    final val ClusterTopKVideoTweetsNodeName: String = "ClusterTopKVideoTweetsSummer"
    final val NormPerTweetWriteNode: String = "NormPerTweetWriteNode"
  }

  def generate[P <: Platform[P]](
    profile: SimClustersTweetProfile,
    uuaEventSource: Producer[P, UnifiedUserAction],
    userInterestedInService: P#Service[Long, ClustersUserIsInterestedIn],
    tweetClusterScoreStore: P#Store[(SimClusterEntity, FullClusterIdBucket), ClustersWithScores],
    tweetTopKClustersStore: P#Store[EntityWithVersion, TopKClustersWithScores],
    clusterTopKTweetsStore: P#Store[FullClusterId, TopKTweetsWithScores],
    clusterTopKHydratedTweetsStore: P#Store[FullClusterId, TopKHydratedTweetsWithScores],
    clusterTopKVideoTweetsStore: P#Store[FullClusterId, TopKTweetsWithScores],
    tweetMetadataService: P#Service[SimClusterEntity, TweetMetadata],
    squaredL2NormSink: P#Sink[(EntityWithVersion, SquaredL2Norm)]
  )(
    implicit jobId: JobId
  ): TailProducer[P, Any] = {

    val userInterestNonEmptyCount = Counter(Group(jobId.get), Name("num_user_interests_non_empty"))
    val userInterestEmptyCount = Counter(Group(jobId.get), Name("num_user_interests_empty"))

    val numClustersCount = Counter(Group(jobId.get), Name("num_clusters"))

    val entityClusterPairCount = Counter(Group(jobId.get), Name("num_entity_cluster_pairs_emitted"))

    val qualifiedFavEvents = uuaEventSource
      .collect {
        case UnifiedUserAction(
              UserIdentifier(Some(userId), _, _, _, _, _, _, _, _, _, _, _, _, _, _),
              TweetInfo(tweetInfo),
              ActionType.ServerTweetFav,
              eventMetadata,
              _,
              _)
            if tweetInfo.actionTweetAuthorInfo.exists(!_.authorId.contains(userId))
              && !isTweetTooOld(tweetInfo.actionTweetId) =>
          (userId, (tweetInfo.actionTweetId, eventMetadata.receivedTimestampMs))
      }
      .observe("num_qualified_favorite_events")

    val entityWithSimClustersProducer = qualifiedFavEvents
      .leftJoin(userInterestedInService)
      .map {
        case (_, ((tweetId, eventTimeMs), userInterestOpt)) =>
          (tweetId, (eventTimeMs, userInterestOpt))
      }
      .flatMap {
        case (tweetId, (eventTimeMs, Some(userInterests))) =>
          userInterestNonEmptyCount.incr()

          val clustersWithScores = SimClustersInterestedInUtil.topClustersWithScores(userInterests)

          numClustersCount.incrBy(clustersWithScores.size)

          val simClusterScoresByHashBucket = clustersWithScores.groupBy {
            case (clusterId, _) => SimClustersHashUtil.clusterIdToBucket(clusterId)
          }

          for {
            (hashBucket, scores) <- simClusterScoresByHashBucket
          } yield {
            entityClusterPairCount.incr()

            val clusterBucket = FullClusterIdBucket(userInterests.knownForModelVersion, hashBucket)

            val scTweetId: SimClusterEntity = SimClusterEntity.TweetId(tweetId)

            (scTweetId, clusterBucket) -> SimClustersInterestedInUtil
              .buildClusterWithScores(
                scores,
                eventTimeMs,
                profile.favScoreThresholdForUserInterest
              )
          }
        case _ =>
          userInterestEmptyCount.incr()
          None
      }
      .observe("entity_cluster_delta_scores")
      .name(NodeName.TweetClusterScoreFlatMapNodeName)
      .sumByKey(tweetClusterScoreStore)(clustersWithScoreMonoid)
      .name(NodeName.TweetClusterScoreSummerNodeName)
      .map {
        case ((simClusterEntity, clusterBucket), (oldValueOpt, deltaValue)) =>
          val updatedClusterIds = deltaValue.clustersToScore.map(_.keySet).getOrElse(Set.empty[Int])

          (simClusterEntity, clusterBucket) -> clustersWithScoreMonoid.plus(
            oldValueOpt
              .map { oldValue =>
                oldValue.copy(
                  clustersToScore =
                    oldValue.clustersToScore.map(_.filterKeys(updatedClusterIds.contains))
                )
              }.getOrElse(clustersWithScoreMonoid.zero),
            deltaValue
          )
      }
      .observe("entity_cluster_updated_scores")
      .name(NodeName.TweetClusterUpdatedScoresFlatMapNodeName)

    val tweetTopK = entityWithSimClustersProducer
      .flatMap {
        case ((simClusterEntity, FullClusterIdBucket(modelVersion, _)), clusterWithScores)
            if simClusterEntity.isInstanceOf[SimClusterEntity.TweetId] =>
          clusterWithScores.clustersToScore
            .map { clustersToScores =>
              val topClustersWithFavScores = clustersToScores.mapValues { scores: Scores =>
                Scores(
                  favClusterNormalized8HrHalfLifeScore =
                    scores.favClusterNormalized8HrHalfLifeScore.filter(
                      _.value >= Configs.scoreThresholdForTweetTopKClustersCache
                    )
                )
              }

              (
                EntityWithVersion(simClusterEntity, modelVersion),
                TopKClustersWithScores(Some(topClustersWithFavScores), None)
              )
            }
        case _ =>
          None

      }
      .observe("tweet_topk_updates")
      .sumByKey(tweetTopKClustersStore)(topKClustersWithScoresMonoid)
      .name(NodeName.TweetTopKNodeName)
      .map {
        case (entityWithVersion, (oldValueOpt, deltaValue)) =>
          val topKClusters = oldValueOpt
            .map {
              topKClustersWithScoresMonoid.plus(_, deltaValue)
            }.getOrElse(deltaValue)

          val favSquaredL2Norm = topKClusters.topClustersByFavClusterNormalizedScore
            .map { clustersToScores =>
              clustersToScores.values
                .flatMap(_.favClusterNormalized8HrHalfLifeScore.map(_.value))
                .map(score => score * score)
                .sum
            }

          (
            entityWithVersion,
            SquaredL2Norm(favSquaredL2Norm, None)
          )
      }
      .write(squaredL2NormSink)
      .name(NodeName.NormPerTweetWriteNode)

    val clusterTopKTweets = entityWithSimClustersProducer
      .flatMap {
        case ((simClusterEntity, FullClusterIdBucket(modelVersion, _)), clusterWithScores) =>
          simClusterEntity match {
            case SimClusterEntity.TweetId(tweetId) =>
              clusterWithScores.clustersToScore
                .map { clustersToScores =>
                  clustersToScores.toSeq.map {
                    case (clusterId, scores) =>
                      val topTweetsByFavScore = Map(
                        tweetId -> Scores(favClusterNormalized8HrHalfLifeScore =
                          scores.favClusterNormalized8HrHalfLifeScore.filter(_.value >=
                            Configs.scoreThresholdForClusterTopKTweetsCache)))

                      (
                        FullClusterId(modelVersion, clusterId),
                        TopKTweetsWithScores(Some(topTweetsByFavScore), None)
                      )
                  }
                }.getOrElse(Nil)
            case _ =>
              Nil
          }
      }
      .observe("cluster_topk_tweets_updates")
      .sumByKey(clusterTopKTweetsStore)(topKTweetsWithScoresMonoid)
      .name(NodeName.ClusterTopKTweetsNodeName)

    val hydratedEntityWithSimClustersProducer = entityWithSimClustersProducer
      .map {
        case ((simClusterEntity, fullClusterId), clusterWithScores) =>
          (simClusterEntity, (fullClusterId, clusterWithScores))
      }
      .leftJoin[TweetMetadata](tweetMetadataService)

    val clusterTopKVideoTweets =
      hydratedEntityWithSimClustersProducer
        .collect {
          case (simClusterEntity, ((fullClusterId, clusterWithScores), Some(tweetMetadata)))
              if tweetMetadata.isHighMediaResolution =>
            ((simClusterEntity, fullClusterId), clusterWithScores)
        }
        .flatMap {
          case ((simClusterEntity, FullClusterIdBucket(modelVersion, _)), clusterWithScores) =>
            simClusterEntity match {
              case SimClusterEntity.TweetId(tweetId) =>
                clusterWithScores.clustersToScore
                  .map { clustersToScores =>
                    clustersToScores.toSeq.map {
                      case (clusterId, scores) =>
                        val topTweetsByFavScore = Map(
                          tweetId -> Scores(favClusterNormalized8HrHalfLifeScore =
                            scores.favClusterNormalized8HrHalfLifeScore.filter(_.value >=
                              Configs.scoreThresholdForClusterTopKTweetsCache)))

                        (
                          FullClusterId(modelVersion, clusterId),
                          TopKTweetsWithScores(Some(topTweetsByFavScore), None)
                        )
                    }
                  }.getOrElse(Nil)
              case _ =>
                Nil
            }
        }
        .observe("cluster_topk_video_tweets_updates")
        .sumByKey(clusterTopKVideoTweetsStore)(topKVideoTweetsWithScoresMonoid)
        .name(NodeName.ClusterTopKVideoTweetsNodeName)

    val clusterTopKTweetsWithAuthor = hydratedEntityWithSimClustersProducer
      .collect {
        case (simClusterEntity, ((fullClusterId, clusterWithScores), Some(tweetMetadata)))
            if tweetMetadata.authorId.isDefined =>
          ((simClusterEntity, fullClusterId, tweetMetadata.authorId.get), clusterWithScores)
      }
      .flatMap {
        case (
              (simClusterEntity, FullClusterIdBucket(modelVersion, _), authorId),
              clusterWithScores) =>
          simClusterEntity match {
            case SimClusterEntity.TweetId(tweetId) =>
              clusterWithScores.clustersToScore
                .map { clustersToScores =>
                  clustersToScores.toSeq.map {
                    case (clusterId, scores) =>
                      val topTweetsByFavScore = Map(HydratedTweet(
                        tweetId,
                        Some(authorId)) -> Scores(favClusterNormalized8HrHalfLifeScore =
                        scores.favClusterNormalized8HrHalfLifeScore.filter(_.value >=
                          Configs.scoreThresholdForClusterTopKTweetsCache)))

                      (
                        FullClusterId(modelVersion, clusterId),
                        TopKHydratedTweetsWithScores(Some(topTweetsByFavScore), None)
                      )
                  }
                }.getOrElse(Nil)
            case _ =>
              Nil
          }
      }
      .observe("cluster_topk_tweets_with_author_updates")
      .sumByKey(clusterTopKHydratedTweetsStore)(topKHydratedTweetsWithScoresMonoid)
      .name(NodeName.ClusterTopKHydratedTweetsNodeName)

    tweetTopK
      .also(clusterTopKTweets)
      .also(clusterTopKVideoTweets)
      .also(clusterTopKTweetsWithAuthor)
  }

  private def isTweetTooOld(tweetId: TweetId): Boolean = {
    SnowflakeId.unixTimeMillisOptFromId(tweetId).exists { millis =>
      System.currentTimeMillis() - millis >= Configs.OldestTweetFavEventTimeInMillis
    }
  }

  private def isTweetTooOldForLight(tweetId: Long): Boolean = {
    SnowflakeId.unixTimeMillisOptFromId(tweetId).exists { millis =>
      System.currentTimeMillis() - millis >= Configs.OldestTweetInLightIndexInMillis
    }
  }

}
