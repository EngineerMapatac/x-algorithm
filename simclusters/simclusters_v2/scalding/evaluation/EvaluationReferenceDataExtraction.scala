package com.twitter.simclusters_v2.scalding.evaluation

import com.twitter.ml.api.constant.SharedFeatures.AUTHOR_ID
import com.twitter.ml.api.constant.SharedFeatures.TIMESTAMP
import com.twitter.ml.api.constant.SharedFeatures.TWEET_ID
import com.twitter.ml.api.constant.SharedFeatures.USER_ID
import com.twitter.ml.api.DailySuffixFeatureSource
import com.twitter.ml.api.DataSetPipe
import com.twitter.ml.api.RichDataRecord
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.scalding_internal.job.analytics_batch.AnalyticsBatchExecution
import com.twitter.scalding_internal.job.analytics_batch.AnalyticsBatchExecutionArgs
import com.twitter.scalding_internal.job.analytics_batch.BatchDescription
import com.twitter.scalding_internal.job.analytics_batch.BatchFirstTime
import com.twitter.scalding_internal.job.analytics_batch.BatchIncrement
import com.twitter.scalding_internal.job.analytics_batch.TwitterScheduledExecutionApp
import com.twitter.simclusters_v2.hdfs_sources.TimelineDataExtractorFixedPathSource
import com.twitter.simclusters_v2.hdfs_sources._
import com.twitter.simclusters_v2.thriftscala.DisplayLocation
import com.twitter.simclusters_v2.thriftscala.ReferenceTweet
import com.twitter.simclusters_v2.thriftscala.ReferenceTweets
import com.twitter.simclusters_v2.thriftscala.TweetLabels
import com.twitter.timelines.prediction.features.common.TimelinesSharedFeatures.IS_LINGER_IMPRESSION
import com.twitter.timelines.prediction.features.common.TimelinesSharedFeatures.SOURCE_AUTHOR_ID
import com.twitter.timelines.prediction.features.common.TimelinesSharedFeatures.SOURCE_TWEET_ID
import com.twitter.timelines.prediction.features.itl.ITLFeatures
import com.twitter.timelines.prediction.features.recap.RecapFeatures
import java.util.TimeZone

object ScheduledTimelinesDataExtractionBatch extends TwitterScheduledExecutionApp {

  val outputPath = "/user/cassowary/processed/tweet_evaluation_reference_set/timelines"

  private val firstTime: String = "2019-03-31"
  private implicit val tz: TimeZone = DateOps.UTC
  private implicit val parser: DateParser = DateParser.default
  private val batchIncrement: Duration = Days(1)

  private val execArgs = AnalyticsBatchExecutionArgs(
    batchDesc = BatchDescription(this.getClass.getName.replace("$", "")),
    firstTime = BatchFirstTime(RichDate(firstTime)),
    lastTime = None,
    batchIncrement = BatchIncrement(batchIncrement)
  )

  override def scheduledJob: Execution[Unit] = AnalyticsBatchExecution(execArgs) {
    implicit dateRange =>
      Execution.withId { implicit uniqueId =>
        Execution.withArgs { args =>
          val defaultSampleRate = 1.0
          val recaps =
            TimelinesEngagementDataExtractor.readTimelinesRecapTweets(
              recapTweets =
                DailySuffixFeatureSource(TimelinesEngagementDataExtractor.RecapTweetHdfsPath).read,
              sampleRate = defaultSampleRate
            )(dateRange)
          val recTweets =
            TimelinesEngagementDataExtractor.readTimelinesRecTweets(
              recTweets =
                DailySuffixFeatureSource(TimelinesEngagementDataExtractor.RecTweetHdfsPath).read,
              sampleRate = defaultSampleRate
            )(dateRange)

          (recaps ++ recTweets).writeDALSnapshotExecution(
            TweetEvaluationTimelinesReferenceSetScalaDataset,
            D.Daily,
            D.Suffix(outputPath),
            D.EBLzo(),
            dateRange.end
          )
        }
      }
  }
}

object AdhocTimelinesDataExtraction extends TwitterExecutionApp {

  @Override
  def job: Execution[Unit] = {
    Execution.withArgs { args =>
      implicit val dateRange: DateRange =
        DateRange.parse(args.list("date"))(DateOps.UTC, DateParser.default)

      val outputDir = args("output_dir")
      val readRecTweet = args.boolean("rectweet")
      val readRecap = args.boolean("recap")
      val sampleRate = args.double("sample_rate")
      val useTsv = args.boolean("output_tsv")

      if (!readRecTweet && !readRecap) {
        throw new IllegalArgumentException("Must read at least some data!")
      }
      val recTweets = if (readRecTweet) {
        println("RecTweets are included in the dataset")
        TimelinesEngagementDataExtractor.readTimelinesRecTweets(
          recTweets =
            DailySuffixFeatureSource(TimelinesEngagementDataExtractor.RecTweetHdfsPath).read,
          sampleRate = sampleRate)(dateRange)
      } else {
        TypedPipe.empty
      }

      val recaps = if (readRecap) {
        println("Recaps are included in the dataset")
        TimelinesEngagementDataExtractor.readTimelinesRecapTweets(
          recapTweets =
            DailySuffixFeatureSource(TimelinesEngagementDataExtractor.RecapTweetHdfsPath).read,
          sampleRate = sampleRate
        )(dateRange)
      } else {
        TypedPipe.empty
      }

      val referenceTweets = recaps ++ recTweets

      if (useTsv) {
        referenceTweets
          .map(t => (t.targetUserId, t.impressedTweets))
          .writeExecution(TypedTsv[(Long, Seq[ReferenceTweet])](outputDir))
      } else {
        referenceTweets
          .writeExecution(TimelineDataExtractorFixedPathSource(outputDir))
      }
    }
  }
}

object TimelinesEngagementDataExtractor {

  val RecapTweetHdfsPath = "/atla/proc2/user/timelines/processed/suggests/recap/data_records"
  val RecTweetHdfsPath = "/atla/proc2/user/timelines/processed/injections/rectweet/data_records"

  def toRecapTweetLabels(record: RichDataRecord): TweetLabels = {
    val isClicked = record.getFeatureValue(RecapFeatures.IS_CLICKED)
    val isFav = record.getFeatureValue(RecapFeatures.IS_FAVORITED)
    val isRT = record.getFeatureValue(RecapFeatures.IS_RETWEETED)
    val isQuoted = record.getFeatureValue(RecapFeatures.IS_QUOTED)
    val isReplied = record.getFeatureValue(RecapFeatures.IS_REPLIED)
    TweetLabels(isClicked, isFav, isRT, isQuoted, isReplied)
  }

  def toRecTweetLabels(record: RichDataRecord): TweetLabels = {
    val isClicked = record.getFeatureValue(ITLFeatures.IS_CLICKED)
    val isFav = record.getFeatureValue(ITLFeatures.IS_FAVORITED)
    val isRT = record.getFeatureValue(ITLFeatures.IS_RETWEETED)
    val isQuoted = record.getFeatureValue(ITLFeatures.IS_QUOTED)
    val isReplied = record.getFeatureValue(ITLFeatures.IS_REPLIED)
    TweetLabels(isClicked, isFav, isRT, isQuoted, isReplied)
  }

  def readTimelinesRecapTweets(
    recapTweets: DataSetPipe,
    sampleRate: Double
  )(
    implicit dateRange: DateRange
  ): TypedPipe[ReferenceTweets] = {

    recapTweets.richRecords
      .sample(sampleRate)
      .filter { record =>
        val isInDateRange = dateRange.contains(RichDate(record.getFeatureValue(TIMESTAMP).toLong))
        val isLingeredImpression = record.getFeatureValue(IS_LINGER_IMPRESSION)
        val isInNetwork =
          record.getFeatureValue(RecapFeatures.PROBABLY_FROM_FOLLOWED_AUTHOR)
        val isRetweet = record.getFeatureValue(RecapFeatures.IS_RETWEET)
        isRetweet && (!isInNetwork) && isInDateRange && isLingeredImpression
      }
      .flatMap { record =>
        for {
          userId <- Option(record.getFeatureValue(USER_ID)).map(_.toLong)
          sourceTweetId <- Option(record.getFeatureValue(SOURCE_TWEET_ID)).map(
            _.toLong
          )
          sourceAuthorId <- Option(record.getFeatureValue(SOURCE_AUTHOR_ID)).map(_.toLong)
          timestamp <- Option(record.getFeatureValue(TIMESTAMP)).map(_.toLong)
          labels = toRecapTweetLabels(record)
        } yield {
          (
            userId,
            Seq(
              ReferenceTweet(
                sourceTweetId,
                sourceAuthorId,
                timestamp,
                DisplayLocation.TimelinesRecap,
                labels))
          )
        }
      }
      .sumByKey
      .map { case (uid, tweetSeq) => ReferenceTweets(uid, tweetSeq) }
  }

  def readTimelinesRecTweets(
    recTweets: DataSetPipe,
    sampleRate: Double
  )(
    implicit dateRange: DateRange
  ): TypedPipe[ReferenceTweets] = {

    recTweets.richRecords
      .sample(sampleRate)
      .filter { record =>
        val isInDateRange = dateRange.contains(RichDate(record.getFeatureValue(TIMESTAMP).toLong))
        val isLingeredImpression = record.getFeatureValue(IS_LINGER_IMPRESSION)

        isInDateRange && isLingeredImpression
      }
      .flatMap { record =>
        for {
          userId <- Option(record.getFeatureValue(USER_ID)).map(_.toLong)
          tweetId <- Option(record.getFeatureValue(TWEET_ID)).map(_.toLong)
          authorId <- Option(record.getFeatureValue(AUTHOR_ID)).map(_.toLong)
          timestamp <- Option(record.getFeatureValue(TIMESTAMP)).map(_.toLong)
          labels = toRecTweetLabels(record)
        } yield {
          (
            userId,
            Seq(
              ReferenceTweet(
                tweetId,
                authorId,
                timestamp,
                DisplayLocation.TimelinesRectweet,
                labels))
          )
        }
      }
      .sumByKey
      .map { case (uid, tweetSeq) => ReferenceTweets(uid, tweetSeq) }
  }
}
