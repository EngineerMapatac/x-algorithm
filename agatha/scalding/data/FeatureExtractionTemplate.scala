package com.twitter.agatha.scalding.data

import com.twitter.agatha.scalding.data.DataLoaderUtil.createUserFeatures
import com.twitter.hub.agatha.thriftscala.UserFeature
import com.twitter.scalding._
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.mediascience.scalding.extensions.MediaScienceBatch
import com.twitter.scalding_internal.job.analytics_batch.AnalyticsBatchExecution
import com.twitter.scalding_internal.job.analytics_batch.TwitterScheduledExecutionApp

import java.util.TimeZone

trait FeatureExtractionConfig {
  final implicit val tz: TimeZone = DateOps.UTC
  val historyDuration: Duration
  val dataLoaders: Set[DataLoader]

  def getUsers(dateRange: DateRange): TypedPipe[Long]

  def writeUserFeatures(userFeatures: TypedPipe[UserFeature], dateRange: DateRange): Execution[Unit]
}

class FeatureExtractionTemplate(config: FeatureExtractionConfig) extends TwitterExecutionApp {
  override def job: Execution[Unit] = Execution.withId { implicit uniqueId =>
    Execution.getArgs.flatMap { args =>
      implicit val tz: TimeZone = DateOps.UTC
      implicit val dp: DateParser = DateParser.default
      val argDateRange = DateRange.parse(args.list("date"))
      val dateRange: DateRange =
        DateRange(argDateRange.start - config.historyDuration, argDateRange.end)

      val userFeatures =
        createUserFeatures(config.dataLoaders, config.getUsers(dateRange), dateRange)
      config.writeUserFeatures(userFeatures, dateRange)
    }
  }
}

class ScheduledFeatureExtractionTemplate(config: FeatureExtractionConfig)
    extends TwitterScheduledExecutionApp {
  override def scheduledJob: Execution[Unit] = Execution.withId { implicit uniqueId =>
    Execution.getConfig.flatMap { execConfig =>
      val userDescription = execConfig.getArgs.required("batchDesc")
      val className = execConfig.getCascadingAppName.getOrElse(getClass.getCanonicalName)
      val batchDesc = s"$className:$userDescription"
      val args = execConfig.getArgs + (("batchDesc", Iterable(batchDesc)))

      AnalyticsBatchExecution(MediaScienceBatch.batchArgs(args)) { stateBirdDateRange: DateRange =>
        val dateRange: DateRange =
          DateRange(stateBirdDateRange.start - config.historyDuration, stateBirdDateRange.end)
        val userFeatures =
          createUserFeatures(config.dataLoaders, config.getUsers(dateRange), dateRange)
        config.writeUserFeatures(userFeatures, dateRange)
      }
    }
  }
}
