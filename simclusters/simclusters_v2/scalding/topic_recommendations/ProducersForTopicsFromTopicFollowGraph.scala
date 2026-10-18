package com.twitter.simclusters_v2.scalding.topic_recommendations

import com.twitter.bijection.Bufferable
import com.twitter.bijection.Injection
import com.twitter.recos.entities.thriftscala._
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.common.Country
import com.twitter.simclusters_v2.common.Language
import com.twitter.simclusters_v2.common.TopicId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.hdfs_sources.DataSources
import com.twitter.simclusters_v2.hdfs_sources.TopProducersForLocaleTopicsFromTopicFollowGraphScalaDataset
import com.twitter.simclusters_v2.scalding.common.matrix.SparseMatrix
import com.twitter.simclusters_v2.scalding.embedding.common.EmbeddingUtil.ProducerId
import com.twitter.simclusters_v2.scalding.embedding.common.ExternalDataSources
import com.twitter.simclusters_v2.thriftscala.UserAndNeighbors
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import java.util.TimeZone

object ProducersForTopicsFromTopicFollowGraphAdhocApp extends AdhocExecutionApp {

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    import ProducersForTopicsFromTopicFollowGraph._
    val outputDirProducersPerTopic = args("output_dir_producers_per_topic")
    val minActiveFollowersForProducer = args.int("minActiveFollowers", 400)
    val maxProducersPerTopicPerLocale = args.int("maxProducersPerTopic", 50)
    val minTopicFollows = args.int("minTopicFollows", 100)

    val topicsFollowedByProducersFollowers = getTopicsFromProducersFollowers(
      DataSources
        .userUserNormalizedGraphSource(dateRange.prepend(Days(7))),
      ExternalDataSources.topicFollowGraphSource,
      ExternalDataSources.userSource,
      ExternalDataSources.inferredUserConsumedLanguageSource,
      minActiveFollowersForProducer,
      minTopicFollows
    )

    sortAndGetTopProducersPerLocaleTopic(
      topicsFollowedByProducersFollowers,
      maxProducersPerTopicPerLocale).writeExecution(TypedTsv(outputDirProducersPerTopic))

  }
}

object ProducersForTopicsFromTopicFollowGraphBatchApp extends ScheduledExecutionApp {
  override val firstTime: RichDate = RichDate("2020-10-01")

  override val batchIncrement: Duration = Days(1)

  private val topProducersForLocaleTopicsPath: String =
    "/user/cassowary/manhattan_sequence_files/top_producers_for_topics_from_topic_follow_graph"

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    import ProducersForTopicsFromTopicFollowGraph._
    val minActiveFollowersForProducer = args.int("minActiveFollowers", 400)
    val maxProducersPerTopicPerLocale = args.int("maxProducersPerTopic", 50)
    val minTopicFollows = args.int("minTopicFollows", 100)

    val topicsFollowedByProducersFollowers = getTopicsFromProducersFollowers(
      DataSources
        .userUserNormalizedGraphSource(dateRange.prepend(Days(7))),
      ExternalDataSources.topicFollowGraphSource,
      ExternalDataSources.userSource,
      ExternalDataSources.inferredUserConsumedLanguageSource,
      minActiveFollowersForProducer,
      minTopicFollows
    )

    sortAndGetTopProducersPerLocaleTopic(
      topicsFollowedByProducersFollowers,
      maxProducersPerTopicPerLocale)
      .map {
        case ((topicId, languageOpt, countryOpt), producersWithScores) =>
          KeyVal(
            SemanticCoreEntityWithLocale(
              entityId = topicId,
              context = Locale(language = languageOpt, country = countryOpt)),
            UserScoreList(producersWithScores.map {
              case (producerId, producerScore) =>
                UserWithScore(userId = producerId, score = producerScore)
            })
          )
      }.writeDALVersionedKeyValExecution(
        TopProducersForLocaleTopicsFromTopicFollowGraphScalaDataset,
        D.Suffix(topProducersForLocaleTopicsPath),
        version = ExplicitEndTime(dateRange.end)
      )
  }
}

object ProducersForTopicsFromTopicFollowGraph {

  implicit val sparseMatrixInj: Injection[
    (ProducerId, Option[Language], Option[Country]),
    Array[Byte]
  ] =
    Bufferable.injectionOf[(ProducerId, Option[Language], Option[Country])]

  def sortAndGetTopProducersPerLocaleTopic(
    producerToTopics: TypedPipe[(ProducerId, (TopicId, Option[Language], Option[Country]), Double)],
    maxProducersPerLocaleTopic: Int
  )(
    implicit uniqueID: UniqueID
  ): TypedPipe[((TopicId, Option[Language], Option[Country]), List[(ProducerId, Double)])] = {
    val numTopicsWithLocales = Stat("num_topics_with_locales")
    producerToTopics
      .map {
        case (producerId, (topicId, languageOpt, countryOpt), score) =>
          ((topicId, languageOpt, countryOpt), Seq((producerId, score)))
      }
      .sumByKey.mapValues { producersList =>
        numTopicsWithLocales.inc()
        producersList.sortBy(-_._2).take(maxProducersPerLocaleTopic).toList
      }.toTypedPipe
  }

  def getTopicsFromProducersFollowers(
    userUserGraph: TypedPipe[UserAndNeighbors],
    followedTopicsToUsers: TypedPipe[(TopicId, UserId)],
    userSource: TypedPipe[(UserId, (Country, Language))],
    userLanguages: TypedPipe[(UserId, Seq[(Language, Double)])],
    minActiveFollowersForProducer: Int,
    minTopicFollows: Int
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): TypedPipe[(ProducerId, (TopicId, Option[Language], Option[Country]), Double)] = {

    val usersFollowingTopics: TypedPipe[UserId] = followedTopicsToUsers.map(_._2).distinct
    val producerToUsersSparseMatrix: SparseMatrix[ProducerId, UserId, Double] =
      TopicsForProducersUtils
        .getProducersToFollowedByUsersSparseMatrix(
          userUserGraph,
          minActiveFollowersForProducer).filterCols(usersFollowingTopics).rowL2Normalize

    val userToTopicsSparseSkinnyMatrix: SparseMatrix[
      UserId,
      (TopicId, Option[Language], Option[Country]),
      Double
    ] =
      TopicsForProducersUtils
        .getFollowedTopicsToUserSparseMatrix(
          followedTopicsToUsers,
          userSource,
          userLanguages,
          minTopicFollows).rowL2Normalize.transpose

    val producersToLocaleTopicsMatrix: SparseMatrix[
      ProducerId,
      (TopicId, Option[Language], Option[Country]),
      Double
    ] =
      producerToUsersSparseMatrix.multiplySparseMatrix(userToTopicsSparseSkinnyMatrix)

    producersToLocaleTopicsMatrix.toTypedPipe
  }
}
