package com.twitter.simclusters_v2.scalding.topic_recommendations
import com.twitter.bijection.Bufferable
import com.twitter.bijection.Injection
import com.twitter.recos.entities.thriftscala._
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.common.Country
import com.twitter.simclusters_v2.common.Language
import com.twitter.simclusters_v2.common.SemanticCoreEntityId
import com.twitter.simclusters_v2.common.TopicId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.hdfs_sources.DataSources
import com.twitter.simclusters_v2.hdfs_sources.TopLocaleTopicsForProducerFromEmScalaDataset
import com.twitter.simclusters_v2.scalding.common.matrix.SparseMatrix
import com.twitter.simclusters_v2.scalding.embedding.common.EmbeddingUtil.ProducerId
import com.twitter.simclusters_v2.scalding.embedding.common.ExternalDataSources
import com.twitter.simclusters_v2.thriftscala.UserAndNeighbors
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import com.twitter.wtf.scalding.jobs.common.EMRunner
import java.util.TimeZone

object TopicsForProducersFromEMAdhocApp extends AdhocExecutionApp {

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    import TopicsForProducersFromEM._
    val outputDirTopicsPerProducer = args("output_dir_topics_per_producer")
    val minActiveFollowersForProducer = args.int("minActiveFollowers", 100)
    val minTopicFollowsThreshold = args.int("minNumTopicFollows", 100)
    val maxTopicsPerProducerPerLocale = args.int("maxTopicsPerProducer", 100)
    val lambda = args.double("lambda", 0.95)

    val numEMSteps = args.int("numEM", 100)

    val topicsFollowedByProducersFollowers: TypedPipe[
      (ProducerId, (TopicId, Option[Language], Option[Country]), Double)
    ] = getTopLocaleTopicsForProducersFromEM(
      DataSources
        .userUserNormalizedGraphSource(dateRange.prepend(Days(7))),
      ExternalDataSources.topicFollowGraphSource,
      ExternalDataSources.userSource,
      ExternalDataSources.inferredUserConsumedLanguageSource,
      minActiveFollowersForProducer,
      minTopicFollowsThreshold,
      lambda,
      numEMSteps
    )

    val topTopicsPerLocaleProducerTsvExec = sortAndGetTopLocaleTopicsPerProducer(
      topicsFollowedByProducersFollowers,
      maxTopicsPerProducerPerLocale
    ).writeExecution(
      TypedTsv(outputDirTopicsPerProducer)
    )

    topTopicsPerLocaleProducerTsvExec
  }
}

object TopicsForProducersFromEMBatchApp extends ScheduledExecutionApp {
  override val firstTime: RichDate = RichDate("2020-07-26")

  override val batchIncrement: Duration = Days(7)

  private val topTopicsPerProducerFromEMPath: String =
    "/user/cassowary/manhattan_sequence_files/top_topics_for_producers_from_em"

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    import TopicsForProducersFromEM._

    val minActiveFollowersForProducer = args.int("minActiveFollowers", 100)

    val minTopicFollowsThreshold = args.int("minNumTopicFollows", 100)

    val maxTopicsPerProducer = args.int("maxTopicsPerProducer", 100)

    val lambda = args.double("lambda", 0.95)

    val numEMSteps = args.int("numEM", 100)

    val topicsFollowedByProducersFollowers = getTopLocaleTopicsForProducersFromEM(
      DataSources
        .userUserNormalizedGraphSource(dateRange.prepend(Days(7))),
      ExternalDataSources.topicFollowGraphSource,
      ExternalDataSources.userSource,
      ExternalDataSources.inferredUserConsumedLanguageSource,
      minActiveFollowersForProducer,
      minTopicFollowsThreshold,
      lambda,
      numEMSteps
    )

    val topLocaleTopicsForProducersFromEMKeyValExec =
      sortAndGetTopLocaleTopicsPerProducer(
        topicsFollowedByProducersFollowers,
        maxTopicsPerProducer
      ).map {
          case ((producerId, languageOpt, countryOpt), topicsWithScores) =>
            KeyVal(
              UserIdWithLocale(
                userId = producerId,
                locale = Locale(language = languageOpt, country = countryOpt)),
              SemanticCoreEntityScoreList(topicsWithScores.map {
                case (topicid, topicScore) =>
                  SemanticEntityScore(SemanticCoreEntity(entityId = topicid), score = topicScore)
              })
            )
        }.writeDALVersionedKeyValExecution(
          TopLocaleTopicsForProducerFromEmScalaDataset,
          D.Suffix(topTopicsPerProducerFromEMPath),
          version = ExplicitEndTime(dateRange.end)
        )
    topLocaleTopicsForProducersFromEMKeyValExec
  }
}

object TopicsForProducersFromEM {

  private val MinProducerTopicScoreThreshold = 0.0

  implicit val sparseMatrixInj: Injection[
    (SemanticCoreEntityId, Option[Language], Option[Country]),
    Array[Byte]
  ] =
    Bufferable.injectionOf[(SemanticCoreEntityId, Option[Language], Option[Country])]

  def sortAndGetTopLocaleTopicsPerProducer(
    producerToTopics: TypedPipe[(ProducerId, (TopicId, Option[Language], Option[Country]), Double)],
    maxTopicsPerProducerPerLocale: Int
  )(
    implicit uniqueID: UniqueID
  ): TypedPipe[((ProducerId, Option[Language], Option[Country]), List[(TopicId, Double)])] = {
    val numProducersWithLocales = Stat("num_producers_with_locales")
    producerToTopics
      .map {
        case (producerId, (topicId, languageOpt, countryOpt), score) =>
          ((producerId, languageOpt, countryOpt), Seq((topicId, score)))
      }.sumByKey.mapValues { topicsList: Seq[(TopicId, Double)] =>
        numProducersWithLocales.inc()
        topicsList
          .filter(_._2 >= MinProducerTopicScoreThreshold).sortBy(-_._2).take(
            maxTopicsPerProducerPerLocale).toList
      }.toTypedPipe
  }

  def getTopLocaleTopicsForProducersFromEM(
    userUserGraph: TypedPipe[UserAndNeighbors],
    followedTopicsToUsers: TypedPipe[(TopicId, UserId)],
    userSource: TypedPipe[(UserId, (Country, Language))],
    userLanguages: TypedPipe[(UserId, Seq[(Language, Double)])],
    minActiveFollowersForProducer: Int,
    minTopicFollowsThreshold: Int,
    lambda: Double,
    numEMSteps: Int
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): TypedPipe[(ProducerId, (TopicId, Option[Language], Option[Country]), Double)] = {

    val producersToUsersMatrix: SparseMatrix[ProducerId, UserId, Double] =
      TopicsForProducersUtils.getProducersToFollowedByUsersSparseMatrix(
        userUserGraph,
        minActiveFollowersForProducer)

    val topicToUsersMatrix: SparseMatrix[
      (TopicId, Option[Language], Option[Country]),
      UserId,
      Double
    ] = TopicsForProducersUtils.getFollowedTopicsToUserSparseMatrix(
      followedTopicsToUsers,
      userSource,
      userLanguages,
      minTopicFollowsThreshold)

    val domainInputModel = producersToUsersMatrix
      .multiplySparseMatrix(topicToUsersMatrix.transpose).toTypedPipe.map {
        case (producerId, (topicId, languageOpt, countryOpt), dotProduct) =>
          ((producerId, languageOpt, countryOpt), Map(topicId -> dotProduct))
      }.sumByKey.toTypedPipe.map {
        case ((producerId, languageOpt, countryOpt), topicsDomainInputMap) =>
          ((languageOpt, countryOpt), (producerId, topicsDomainInputMap))
      }

    val backgroundModel = topicToUsersMatrix.rowL1Norms.map {
      case ((topicId, languageOpt, countryOpt), numFollowersOfTopic) =>
        ((languageOpt, countryOpt), Map(topicId -> numFollowersOfTopic))
    }.sumByKey

    val resultsFromEMForEachLocale = domainInputModel.hashJoin(backgroundModel).flatMap {
      case (
            (languageOpt, countryOpt),
            ((producerId, domainInputTopicFollowersMap), backgroundModelTopicFollowersMap)) =>
        val emScoredTopicsForEachProducerPerLocale = EMRunner.estimateDomainModel(
          domainInputTopicFollowersMap,
          backgroundModelTopicFollowersMap,
          lambda,
          numEMSteps)

        emScoredTopicsForEachProducerPerLocale.map {
          case (topicId, topicScore) =>
            (producerId, (topicId, languageOpt, countryOpt), topicScore)
        }
    }
    resultsFromEMForEachLocale
  }
}
