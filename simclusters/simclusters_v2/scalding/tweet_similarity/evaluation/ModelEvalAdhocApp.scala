package com.twitter.simclusters_v2.scalding.tweet_similarity.evaluation

import com.twitter.ml.api.Feature.Continuous
import com.twitter.ml.api.DailySuffixFeatureSource
import com.twitter.ml.api.DataSetPipe
import com.twitter.ml.api.RichDataRecord
import com.twitter.scalding._
import com.twitter.scalding.typed.TypedPipe
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.simclusters_v2.tweet_similarity.TweetSimilarityFeatures
import com.twitter.twml.runtime.scalding.TensorflowBatchPredictor
import java.util.TimeZone

object ModelEvalAdhocApp extends TwitterExecutionApp {
  implicit val timeZone: TimeZone = DateOps.UTC
  implicit val dateParser: DateParser = DateParser.default

  def getPredictor(modelName: String, modelSource: String): TensorflowBatchPredictor = {
    val defaultInputNode = "request:0"
    val defaultOutputNode = "response:0"
    TensorflowBatchPredictor(modelName, modelSource, defaultInputNode, defaultOutputNode)
  }

  def getPrediction(
    dataset: DataSetPipe,
    batchPredictor: TensorflowBatchPredictor
  ): TypedPipe[(Long, Long, Boolean, Double, Double)] = {
    val featureContext = dataset.featureContext
    val predictionFeature = new Continuous("output")

    batchPredictor
      .predict(dataset.records)
      .map {
        case (originalDataRecord, predictedDataRecord) =>
          val prediction = new RichDataRecord(predictedDataRecord, featureContext)
            .getFeatureValue(predictionFeature).toDouble
          val richDataRecord = new RichDataRecord(originalDataRecord, featureContext)
          (
            richDataRecord.getFeatureValue(TweetSimilarityFeatures.QueryTweetId).toLong,
            richDataRecord.getFeatureValue(TweetSimilarityFeatures.CandidateTweetId).toLong,
            richDataRecord.getFeatureValue(TweetSimilarityFeatures.Label).booleanValue,
            richDataRecord.getFeatureValue(TweetSimilarityFeatures.CosineSimilarity).toDouble,
            prediction
          )
      }
  }

  override def job: Execution[Unit] =
    Execution.withId { implicit uniqueId =>
      Execution.withArgs { args: Args =>
        implicit val dateRange: DateRange = DateRange.parse(args.list("date"))
        val outputPath: String = args("output_path")
        val dataset: DataSetPipe = DailySuffixFeatureSource(args("dataset_path")).read
        val modelSource: String = args("model_path")
        val modelName: String = "tweet_similarity"

        getPrediction(dataset, getPredictor(modelName, modelSource))
          .writeExecution(TypedTsv[(Long, Long, Boolean, Double, Double)](outputPath))
      }
    }
}
