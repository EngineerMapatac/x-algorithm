package com.twitter.simclusters_v2.scalding.offline_data_conversion

import com.twitter.scalding.Args
import com.twitter.scalding.DateOps
import com.twitter.scalding.DateParser
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.Duration
import com.twitter.scalding.Execution
import com.twitter.scalding.Months
import com.twitter.scalding.RichDate
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import java.util.TimeZone
import com.twitter.scalding_internal.dalv2.DALWrite.D
import com.twitter.scalding_internal.dalv2.DALWrite.WriteExtension
import com.twitter.scalding_internal.dalv2.remote_access.AllowCrossClusterSameDC
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.scalding_internal.job.analytics_batch.AnalyticsBatchExecution
import com.twitter.scalding_internal.job.analytics_batch.AnalyticsBatchExecutionArgs
import com.twitter.scalding_internal.job.analytics_batch.BatchDescription
import com.twitter.scalding_internal.job.analytics_batch.BatchFirstTime
import com.twitter.scalding_internal.job.analytics_batch.BatchIncrement
import com.twitter.scalding_internal.job.analytics_batch.TwitterScheduledExecutionApp
import com.twitter.simclusters_v2.hdfs_sources.SimclustersV2KnownFor20M145K2020ScalaDataset
import com.twitter.simclusters_v2.thriftscala.ClustersUserIsKnownFor

trait KnownFor20M145K2020FlattenBaseJob {
  def runJob(outputPath: String)(implicit dateRange: DateRange, tz: TimeZone): Execution[Unit] = {
    val knownForSimclusters: TypedPipe[KeyVal[Long, ClustersUserIsKnownFor]] = {
      DAL
        .readMostRecentSnapshot(
          SimclustersV2KnownFor20M145K2020ScalaDataset,
          dateRange.embiggen(Months(6)))
        .withRemoteReadPolicy(AllowCrossClusterSameDC)
        .toTypedPipe
    }
    val flattenedKnownForSimclusters =
      KnownFor20M145K2020FlattenTransform.flattenAndHydrateKnownForUserClusters(knownForSimclusters)
    flattenedKnownForSimclusters
      .shard(1).writeDALSnapshotExecution(
        dataset = FlattenedSimclustersV2KnownFor20M145K2020ScalaDataset,
        updateStep = D.Daily(1),
        pathLayout = D.Suffix(outputPath),
        fmt = D.Parquet,
        dateRange.end
      )
  }
}

object KnownFor20M145K2020FlattenAdhocJob
    extends TwitterExecutionApp
    with KnownFor20M145K2020FlattenBaseJob {
  override def job: Execution[Unit] = Execution.withArgs { args =>
    implicit val tz: TimeZone = DateOps.UTC
    implicit val dp: DateParser = DateParser.default
    implicit val dateRange: DateRange =
      DateRange.parse(args("date"))
    val outputPath = args.getOrElse(
      "outputPath",
      "tmp/" + FlattenedSimclustersV2KnownFor20M145K2020ScalaDataset.logicalName)
    runJob(outputPath)
  }
}

object KnownFor20M145K2020FlattenProdJob
    extends TwitterScheduledExecutionApp
    with KnownFor20M145K2020FlattenBaseJob {
  private implicit val tz: TimeZone = DateOps.UTC
  private implicit val dp: DateParser = DateParser.default

  private def batchIncrement: Duration = Days(1)

  private def executionArgs(args: Args): AnalyticsBatchExecutionArgs = {
    AnalyticsBatchExecutionArgs(
      batchDesc = BatchDescription(this.getClass.getName),
      firstTime = BatchFirstTime(RichDate(args("firstTime"))),
      lastTime = None,
      batchIncrement = BatchIncrement(batchIncrement)
    )
  }

  override def scheduledJob: Execution[Unit] = {
    getArgs.flatMap { args =>
      val outputPath = args.getOrElse(
        "outputPath",
        FlattenedSimclustersV2KnownFor20M145K2020ScalaDataset.logicalName
      )
      AnalyticsBatchExecution(executionArgs(args)) { implicit dateRange: DateRange =>
        runJob(outputPath)
      }
    }
  }
}
