package com.twitter.simclusters_v2.scalding.topic_recommendations

import com.twitter.escherbird.scalding.source.FullMetadataSource
import com.twitter.interests_ds.jobs.interests_service.UserTopicRelationSnapshotScalaDataset
import com.twitter.interests.thriftscala.InterestRelationType
import com.twitter.interests.thriftscala.UserInterestsRelationSnapshot
import com.twitter.recos.entities.thriftscala.SemanticCoreEntity
import com.twitter.recos.entities.thriftscala.SemanticCoreEntityScoreList
import com.twitter.recos.entities.thriftscala.SemanticEntityScore
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.common.SemanticCoreEntityId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.hdfs_sources.SimilarTopicsFromTopicFollowGraphScalaDataset
import com.twitter.simclusters_v2.scalding.common.matrix.SparseRowMatrix
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import java.util.TimeZone

object SimilarTopicsFromTopicFollowGraphScheduledApp extends ScheduledExecutionApp {
  import SimilarTopics._

  private val outputPath: String =
    "/user/cassowary/manhattan_sequence_files/similar_topics_from_topics_follow_graph"

  override def firstTime: RichDate = RichDate("2020-05-07")

  override def batchIncrement: Duration = Days(7)

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    val numSimilarTopics = args.int("numSimilarTopics", default = 100)
    val scoreThreshold = args.double("scoreThreshold", default = 0.01)

    val numOutputTopics = Stat("NumOutputTopics")

    computeSimilarTopics(
      getExplicitFollowedTopics,
      getFollowableTopics,
      numSimilarTopics,
      scoreThreshold)
      .map {
        case (topicId, similarTopics) =>
          numOutputTopics.inc()
          KeyVal(
            topicId,
            SemanticCoreEntityScoreList(similarTopics.map {
              case (similarTopicId, score) =>
                SemanticEntityScore(SemanticCoreEntity(similarTopicId), score)
            }))
      }
      .writeDALVersionedKeyValExecution(
        SimilarTopicsFromTopicFollowGraphScalaDataset,
        D.Suffix(outputPath),
        version = ExplicitEndTime(dateRange.end)
      )
  }

}

object SimilarTopicsFromTopicFollowGraphAdhocApp extends AdhocExecutionApp {
  import SimilarTopics._

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    val numSimilarTopics = args.int("numSimilarTopics", default = 100)
    val scoreThreshold = args.double("scoreThreshold", default = 0.01)

    val numOutputTopics = Stat("NumOutputTopics")

    computeSimilarTopics(
      getExplicitFollowedTopics,
      getFollowableTopics,
      numSimilarTopics,
      scoreThreshold)
      .map {
        case (topicId, similarTopics) =>
          numOutputTopics.inc()
          topicId -> similarTopics
            .collect {
              case (similarTopic, score) if similarTopic != topicId =>
                s"$similarTopic:$score"
            }
            .mkString(",")
      }
      .writeExecution(
        TypedTsv("/user/cassowary/adhoc/topic_recos/similar_topics")
      )
  }

}

object SimilarTopics {

  val UTTDomain: Long = 131L

  val FollowableTag: String = "utt:followable_topic"

  def getFollowableTopics(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): TypedPipe[SemanticCoreEntityId] = {
    val NumFollowableTopics = Stat("NumFollowableTopics")

    TypedPipe
      .from(
        new FullMetadataSource("/atla/proc" + FullMetadataSource.DefaultHdfsPath)()(
          dateRange.embiggen(Days(7))))
      .flatMap {
        case fullMetadata if fullMetadata.domainId == UTTDomain =>
          for {
            basicMetadata <- fullMetadata.basicMetadata
            indexableFields <- basicMetadata.indexableFields
            tags <- indexableFields.tags
            if tags.contains(FollowableTag)
          } yield {
            NumFollowableTopics.inc()
            fullMetadata.entityId
          }
        case _ => None
      }
      .forceToDisk
  }

  def getExplicitFollowedTopics(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): TypedPipe[(UserId, Map[SemanticCoreEntityId, Double])] = {

    DAL
      .readMostRecentSnapshotNoOlderThan(UserTopicRelationSnapshotScalaDataset, Days(7))
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .collect {
        case userInterestsRelationSnapshot: UserInterestsRelationSnapshot
            if userInterestsRelationSnapshot.interestType == "UTT" &&
              userInterestsRelationSnapshot.relation == InterestRelationType.Followed =>
          (
            userInterestsRelationSnapshot.userId,
            Map(userInterestsRelationSnapshot.interestId -> 1.0))
      }
      .sumByKey
  }

  def computeSimilarTopics(
    userTopicsFollowGraph: TypedPipe[(UserId, Map[SemanticCoreEntityId, Double])],
    followableTopics: TypedPipe[SemanticCoreEntityId],
    numSimilarTopics: Int,
    scoreThreshold: Double
  ): TypedPipe[(SemanticCoreEntityId, Seq[(SemanticCoreEntityId, Double)])] = {
    val userTopicFollowGraph =
      SparseRowMatrix[UserId, SemanticCoreEntityId, Double](
        userTopicsFollowGraph,
        isSkinnyMatrix = true)
        .filterCols(followableTopics)
        .colL2Normalize
        .forceToDisk(numShardsOpt = Some(10))

    userTopicFollowGraph
      .transposeAndMultiplySkinnySparseRowMatrix(userTopicFollowGraph)
      .filter { (i, j, v) =>
        i != j && v > scoreThreshold
      }
      .sortWithTakePerRow(numSimilarTopics)(Ordering.by(-_._2))
  }

}
