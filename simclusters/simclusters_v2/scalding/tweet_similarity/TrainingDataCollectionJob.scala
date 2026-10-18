package com.twitter.simclusters_v2.scalding.tweet_similarity

import com.twitter.dal.client.dataset.TimePartitionedDALDataset
import com.twitter.ml.api.DataRecord
import com.twitter.ml.api.DataSetPipe
import com.twitter.scalding._
import com.twitter.scalding.typed.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.Proc3Atla
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.simclusters_v2.hdfs_sources.TweetSimilarityUnhydratedPairsSource
import com.twitter.simclusters_v2.scalding.common.LogFavBasedPersistentTweetEmbeddingMhExportSource
import com.twitter.simclusters_v2.scalding.tweet_similarity.TweetPairLabelCollectionUtil.FeaturedTweet
import com.twitter.simclusters_v2.thriftscala.LabelledTweetPairs
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import java.util.TimeZone

object TrainingDataCollectionJob {
  val LookbackDays = 2
  val testLookbackHours = 2
  val testRatio = 0.1

  def getHydratedDataPipe(
    dateRange: DateRange,
    useAuthorFeatures: Boolean,
    unhydratedPairs: TypedPipe[LabelledTweetPairs]
  )(
    implicit timeZone: TimeZone
  ): DataSetPipe = {

    val persistentEmbeddingRecords =
      TypedPipe.from(new LogFavBasedPersistentTweetEmbeddingMhExportSource(range = dateRange))

    val tweetAuthorPairs =
      TweetPairLabelCollectionUtil.getTweetAuthorPairs(dateRange.prepend(Days(LookbackDays)))

    val labelledPairs = unhydratedPairs
      .map { labelledPair =>
        (
          FeaturedTweet(
            labelledPair.queryFeaturedTweet.tweetId,
            labelledPair.queryFeaturedTweet.timestamp,
            None,
            None),
          FeaturedTweet(
            labelledPair.candidateFeaturedTweet.tweetId,
            labelledPair.candidateFeaturedTweet.timestamp,
            None,
            None),
          labelledPair.label
        )
      }

    TweetPairFeatureHydrationUtil.getDataSetPipeWithFeatures(
      labelledPairs,
      persistentEmbeddingRecords,
      tweetAuthorPairs,
      useAuthorFeatures)
  }

  def getTrainTestExec(
    dataSetPipe: DataSetPipe,
    splitBy: Option[String],
    trainDataset: TimePartitionedDALDataset[DataRecord],
    testDataset: TimePartitionedDALDataset[DataRecord],
    outputPath: String
  )(
    implicit timeZone: TimeZone,
    dateRange: DateRange
  ): Execution[Unit] = {
    splitBy match {
      case Some("time") =>
        TrainingDataCollectionUtil.getTrainTestByTimeExec(
          dataSetPipe,
          dateRange.end - Hours(testLookbackHours),
          trainDataset,
          testDataset,
          outputPath)(dateRange)
      case Some("query_tweet") =>
        TrainingDataCollectionUtil.getTrainTestByQueryExec(
          dataSetPipe,
          testRatio,
          trainDataset,
          testDataset,
          outputPath)(dateRange)
      case _ =>
        TrainingDataCollectionUtil.getTrainTestByQueryExec(
          dataSetPipe,
          0.0,
          trainDataset,
          testDataset,
          outputPath)(dateRange)
    }
  }
}

object TrainingDataCollectionAdhocApp extends TwitterExecutionApp {
  implicit val timeZone: TimeZone = DateOps.UTC
  implicit val dateParser: DateParser = DateParser.default

  override def job: Execution[Unit] =
    Execution.withId { implicit uniqueId =>
      Execution.withArgs { args: Args =>
        implicit val dateRange: DateRange = DateRange.parse(args.list("date"))
        val useAuthorFeatures: Boolean = args.boolean("use_author_features")
        val inputPath: String = args("input_path")
        val outputPath: String = args("output_path")
        val splitBy: Option[String] = args.optional("split_by")

        val labelledPairs = TypedPipe
          .from(TweetSimilarityUnhydratedPairsSource(inputPath, dateRange))

        val dataSetPipe = TrainingDataCollectionJob.getHydratedDataPipe(
          dateRange,
          useAuthorFeatures,
          labelledPairs
        )
        TrainingDataCollectionJob.getTrainTestExec(
          dataSetPipe,
          splitBy,
          TweetSimilarityTrainDatarecords30MinJavaDataset,
          TweetSimilarityTestDatarecords30MinJavaDataset,
          outputPath
        )
      }
    }
}

object TrainingDataCollection30MinScheduledApp extends ScheduledExecutionApp {

  private val outputPath: String =
    "/user/cassowary/processed/tweet_similarity/training_data_30min"

  override def batchIncrement: Duration = Hours(24)

  override def firstTime: RichDate = RichDate("2020-03-26")

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    val useAuthorFeatures: Boolean = args.boolean("use_author_features")
    val splitBy: Option[String] = args.optional("split_by")

    val unhydratedPairs = DAL
      .read(TweetSimilarityUnhydratedPairs30MinScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(Proc3Atla))
      .toTypedPipe

    val dataSetPipe = TrainingDataCollectionJob.getHydratedDataPipe(
      dateRange,
      useAuthorFeatures,
      unhydratedPairs
    )
    TrainingDataCollectionJob.getTrainTestExec(
      dataSetPipe,
      splitBy,
      TweetSimilarityTrainDatarecords30MinJavaDataset,
      TweetSimilarityTestDatarecords30MinJavaDataset,
      outputPath)
  }
}

object TrainingDataCollection120MinScheduledApp extends ScheduledExecutionApp {

  private val outputPath: String =
    "/user/cassowary/processed/tweet_similarity/training_data_120min"

  override def batchIncrement: Duration = Hours(24)

  override def firstTime: RichDate = RichDate("2020-03-26")

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    val useAuthorFeatures: Boolean = args.boolean("use_author_features")
    val splitBy: Option[String] = args.optional("split_by")

    val unhydratedPairs = DAL
      .read(TweetSimilarityUnhydratedPairs120MinScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(Proc3Atla))
      .toTypedPipe

    val dataSetPipe = TrainingDataCollectionJob.getHydratedDataPipe(
      dateRange,
      useAuthorFeatures,
      unhydratedPairs
    )

    TrainingDataCollectionJob.getTrainTestExec(
      dataSetPipe,
      splitBy,
      TweetSimilarityTrainDatarecords120MinJavaDataset,
      TweetSimilarityTestDatarecords120MinJavaDataset,
      outputPath)
  }
}
