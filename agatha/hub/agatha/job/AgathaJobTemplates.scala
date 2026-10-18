package com.twitter.hub.agatha.job

import java.util.TimeZone

import com.twitter.hub.agatha.builder.BuilderApp
import com.twitter.hub.agatha.labels.LabelManager
import com.twitter.hub.agatha.predict.PredictionApp
import com.twitter.hub.agatha.predict.UserGraphGrouping
import com.twitter.hub.agatha.thriftscala._
import com.twitter.hub.pmi.PMI.PMIParams
import com.twitter.hub.pmi.PMI.RegularizationParams
import com.twitter.hub.pmi.math.MathParams
import com.twitter.mediascience.scalding.extensions.MediaScienceBatch
import com.twitter.scalding._
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.scalding_internal.job.analytics_batch.AnalyticsBatchExecution
import com.twitter.scalding_internal.job.analytics_batch.TwitterScheduledExecutionApp

object DefaultParams {
  val defaultMathParams = MathParams(
    pseudoCounts = 10000.0,
    z = 2.0,
    positive = false,
    pmiDiscount = 0,
    minAbsPMI = 0.01
  )

  val defaultPMIParams: PMIParams = PMIParams(
    RegularizationParams(
      Some(20),
      None,
      None
    ),
    RegularizationParams(
      None,
      None,
      None
    )
  )
}

trait AgathaJobConfig extends PredictJobConfig with BuildJobConfig

trait BuildJobConfig extends BaseJobConfig {
  val pmiParams: PMIParams = DefaultParams.defaultPMIParams

  def writeModelledFeatures(
    modelledFeature: TypedPipe[ModelledFeature],
    metadata: ModelledFeatureLabelMetadata,
    dateRange: DateRange
  ): Execution[Unit]
}

trait PredictJobConfig extends BaseJobConfig {
  def readModelledFeatures(
    dateRange: DateRange
  )(
    implicit mode: Mode
  ): Map[TypedPipe[ModelledFeature], ModelledFeatureLabelMetadata]

  def outputPredictions(
    pmiSet: TypedPipe[((UserGraphGrouping, String), Double)],
    dateRange: DateRange
  ): Execution[Unit]
}

trait BaseJobConfig {
  final implicit val tz: TimeZone = DateOps.UTC
  val historyDuration: Duration
  val labelManager: LabelManager
  val labelNames: Set[String]
  val mathParams: MathParams = DefaultParams.defaultMathParams

  def readUserFeatures(dateRange: DateRange): TypedPipe[UserFeature]
}

object BaseBuildJob {
  def apply(
    jobConfig: BuildJobConfig,
    passedDateRange: DateRange
  )(
    implicit uniqueId: UniqueID
  ): Execution[Unit] = {
    implicit val dateRange: DateRange =
      DateRange(passedDateRange.start - jobConfig.historyDuration, passedDateRange.end)

    val features = jobConfig.readUserFeatures(dateRange)
    val labelManager = jobConfig.labelManager
    val labelNames = jobConfig.labelNames
    val pmiParams = jobConfig.pmiParams
    val mathParams = jobConfig.mathParams

    val pipe = BuilderApp(features, labelManager, labelNames, pmiParams, mathParams, dateRange)
    val labelDateRange = LabelDateRange(dateRange.start.timestamp, dateRange.end.timestamp)
    val metadata = ModelledFeatureLabelMetadata(
      labelNames.map(name => (name, labelDateRange)).toMap)

    jobConfig.writeModelledFeatures(
      pipe.withDescription("write modelled features"),
      metadata,
      dateRange)
  }
}

class BuildJobTemplate(jobConfig: BuildJobConfig) extends TwitterExecutionApp {
  override def job: Execution[Unit] = Execution.withId { implicit uniqueId =>
    Execution.getArgs.flatMap { args =>
      implicit val tz: TimeZone = jobConfig.tz
      implicit val dp: DateParser = DateParser.default
      val argDateRange = DateRange.parse(args.list("date"))

      BaseBuildJob(jobConfig, argDateRange)
    }
  }
}

class ScheduledBuildJobTemplate(jobConfig: BuildJobConfig) extends TwitterScheduledExecutionApp {
  override def scheduledJob: Execution[Unit] = Execution.withId { implicit uniqueId =>
    Execution.getConfig.flatMap { execConfig =>
      val userDescription = execConfig.getArgs.required("batchDesc")
      val className = execConfig.getCascadingAppName.getOrElse(getClass.getCanonicalName)
      val batchDesc = s"$className:$userDescription"
      val args = execConfig.getArgs + (("batchDesc", Iterable(batchDesc)))

      AnalyticsBatchExecution(MediaScienceBatch.batchArgs(args)) { stateBirdDateRange: DateRange =>
        BaseBuildJob(jobConfig, stateBirdDateRange)
      }
    }
  }
}

object BasePredictJob {
  def apply(
    jobConfig: PredictJobConfig,
    passedDateRange: DateRange
  )(
    implicit uniqueId: UniqueID
  ): Execution[Unit] = {
    Execution.getConfigMode.flatMap {
      case (_, mode) =>
        implicit val dateRange: DateRange =
          DateRange(passedDateRange.start - jobConfig.historyDuration, passedDateRange.end)

        val labelManager = jobConfig.labelManager
        val features = jobConfig.readUserFeatures(dateRange)
        val modelMetadataMap = jobConfig.readModelledFeatures(dateRange)(mode)
        val mathParams = jobConfig.mathParams

        val pmiSet = PredictionApp(features, modelMetadataMap, labelManager, mathParams)
        jobConfig.outputPredictions(pmiSet, dateRange)
    }
  }
}

class PredictJobTemplate(jobConfig: PredictJobConfig) extends TwitterExecutionApp {
  override def job: Execution[Unit] = Execution.withId { implicit uniqueId =>
    Execution.getArgs.flatMap { args =>
      implicit val tz: TimeZone = jobConfig.tz
      implicit val dp: DateParser = DateParser.default
      val argDateRange = DateRange.parse(args.list("date"))
      BasePredictJob(jobConfig, argDateRange)
    }
  }
}

class ScheduledPredictJobTemplate(jobConfig: PredictJobConfig)
    extends TwitterScheduledExecutionApp {
  override def scheduledJob: Execution[Unit] = Execution.withId { implicit uniqueId =>
    Execution.getConfig.flatMap { execConfig =>
      val userDescription = execConfig.getArgs.required("batchDesc")
      val className = execConfig.getCascadingAppName.getOrElse(getClass.getCanonicalName)
      val batchDesc = s"$className:$userDescription"
      val args = execConfig.getArgs + (("batchDesc", Iterable(batchDesc)))

      AnalyticsBatchExecution(MediaScienceBatch.batchArgs(args)) { stateBirdDateRange: DateRange =>
        BasePredictJob(jobConfig, stateBirdDateRange)
      }
    }
  }
}
