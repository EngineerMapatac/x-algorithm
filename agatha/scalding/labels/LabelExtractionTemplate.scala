package com.twitter.agatha.scalding.labels

import com.twitter.dal.client.dataset.SnapshotDALDataset
import com.twitter.hub.agatha.thriftscala.UserLabel
import com.twitter.mediascience.scalding.extensions.MediaScienceBatch
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.scalding_internal.job.analytics_batch.AnalyticsBatchExecution
import com.twitter.scalding_internal.job.analytics_batch.TwitterScheduledExecutionApp
import java.util.TimeZone

case class LabelledUsers(
  userLabels: TypedPipe[UserLabel],
  labelName: String,
  dataset: SnapshotDALDataset[UserLabel])

trait LabelExtractionConfig {
  final implicit val tz: TimeZone = DateOps.UTC
  val historyDuration: Duration

  def generateLabels(dateRange: DateRange): Execution[Seq[LabelledUsers]]

  def writeLabels(users: Execution[Seq[LabelledUsers]], dateRange: DateRange): Execution[Unit] = {
    users.flatMap { labelledUsers: Seq[LabelledUsers] =>
      val writeExecs = labelledUsers.map(labels =>
        labels.userLabels.forceToDisk
          .map(identity).writeDALSnapshotExecution(
            labels.dataset,
            D.Daily,
            D.Suffix("labels/" + labels.labelName),
            D.Parquet,
            dateRange.end
          ))
      Execution.sequence(writeExecs).unit
    }
  }
}

class LabelExtractionTemplate(config: LabelExtractionConfig) extends TwitterExecutionApp {
  override def job: Execution[Unit] = Execution.withId { implicit uniqueId =>
    Execution.getArgs.flatMap { args =>
      implicit val tz: TimeZone = config.tz
      implicit val dp: DateParser = DateParser.default
      val argDateRange = DateRange.parse(args.list("date"))
      val dateRange: DateRange =
        DateRange(argDateRange.start - config.historyDuration, argDateRange.end)

      val userFeatures = config.generateLabels(dateRange)
      config.writeLabels(userFeatures, dateRange)
    }
  }
}

class ScheduledLabelExtractionTemplate(config: LabelExtractionConfig)
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
        val userFeatures = config.generateLabels(dateRange)
        config.writeLabels(userFeatures, dateRange)
      }
    }
  }
}
