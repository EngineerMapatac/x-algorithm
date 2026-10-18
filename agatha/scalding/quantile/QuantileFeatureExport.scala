package com.twitter.agatha.scalding.quantile

import java.util.TimeZone
import com.twitter.agatha.scalding.AgathaQuantileFeaturesScalaDataset
import com.twitter.agatha.scalding.util.GetActiveUsers
import com.twitter.hub.agatha.thriftscala.FeatureClass
import com.twitter.mediascience.scalding.extensions.MediaScienceBatch
import com.twitter.ml.api.Feature.Binary
import com.twitter.ml.api.constant.SharedFeatures
import com.twitter.ml.api.util.FDsl._
import com.twitter.ml.api.DataRecord
import com.twitter.ml.api.DataSetPipe
import com.twitter.ml.api.FeatureContext
import com.twitter.scalding._
import com.twitter.scalding.thrift.macros.Macros._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.dalv2.dataset.DALWrite._
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.scalding_internal.job.analytics_batch.AnalyticsBatchExecution
import com.twitter.scalding_internal.job.analytics_batch.TwitterScheduledExecutionApp

object BaseQuantileFeatureExport {
  type FeatureKey = (FeatureClass, String, Double)
  val roundingPrecision = 5
  val defaultCutoffs = Seq(0.1, 0.01, 0.001, 0.0001)
  val keepQuantiles = Set(0.9)
  private val baseFeatureContext = new FeatureContext(SharedFeatures.USER_ID)

  private[quantile] def createCutoffFeatureMapExec(
    quantileFeatures: TypedPipe[(FeatureKey, (Long, Double))],
    cutoffList: Seq[Double]
  ): Execution[Map[FeatureKey, Map[Double, Binary]]] = {
    quantileFeatures.keys.distinct
      .withDescription("Create Map for feature context")
      .map {
        case (featureClass, label, quantile) =>
          val feature = featureClass.source.name
          val subclass = if (featureClass.subclass.isEmpty) "none" else featureClass.subclass
          val cutoffMap = cutoffList
            .map(cutoff =>
              Map(cutoff -> new Binary(
                s"agatha.$label.$feature.$subclass.quantile_${quantile}_top$cutoff"))).reduce(
              _ ++ _)
          ((featureClass, label, quantile), cutoffMap)
      }
      .toIterableExecution
      .map(_.toMap)
  }

  private[quantile] def createRoundingPrecisionCumulativeHistogram(
    users: TypedPipe[(FeatureKey, (Long, Double))],
    precision: Int
  ): TypedPipe[(FeatureKey, (Double, Long))] = {
    users
      .map {
        case (key, (_, score)) =>
          (key, BigDecimal(score).setScale(precision, BigDecimal.RoundingMode.HALF_UP).toDouble)
      }
      .asKeys
      .group
      .withDescription("Create n-precision buckets and calculate size for each feature.")
      .size
      .map { case ((key, roundedScore), size) => (key, (roundedScore, size)) }
      .group
      .withDescription("Create cumulative counts for each score in decreasing order.")
      .sorted
      .reverse
      .scanLeft((0.0, 0L)) { (prev, current) =>
        (current._1, prev._2 + current._2)
      }
      .toTypedPipe
  }

  private[quantile] def createPerFeatureThresholdMap(
    filteredUsers: TypedPipe[(FeatureKey, (Long, Double))],
    cutoffMap: Map[FeatureKey, Map[Double, Binary]]
  ): TypedPipe[(FeatureKey, Map[Long, Binary])] = {
    filteredUsers.group
      .withDescription("Calculate # users for each feature given a set of % thresholds")
      .size
      .map {
        case ((key, count)) =>
          (
            key,
            cutoffMap(key).map {
              case (threshold, binaryFeature) => ((count * threshold).toLong, binaryFeature)
            })
      }
  }

  private[quantile] def determinePerFeatureCutoffScores(
    filteredUsers: TypedPipe[(FeatureKey, (Long, Double))],
    featureCutoffs: TypedPipe[(FeatureKey, Map[Long, Binary])],
    precision: Int
  ): TypedPipe[(FeatureKey, (Double, String))] = {
    createRoundingPrecisionCumulativeHistogram(filteredUsers, precision).group
      .withDescription("Drop rounded scores with cumulative users < threshold")
      .join(featureCutoffs)
      .flatMapValues {
        case ((roundedScore, cumulativeCount), thresholdMap) =>
          thresholdMap.toSeq.collect {
            case (threshold, binary) if threshold <= cumulativeCount => (roundedScore, binary)
          }
      }
      .map {
        case (key, (roundedScore, binary)) => ((key, binary.getFeatureName), roundedScore)
      }
      .group
      .withDescription("And find highest remaining rounded score for each threshold.")
      .max
      .map { case ((key, binary), roundedScore) => (key, (roundedScore, binary)) }
  }

  def apply(argDateRange: DateRange): Execution[Unit] = {
    implicit val tz: TimeZone = DateOps.UTC
    implicit val dateRange = DateRange(argDateRange.start - Days(10), argDateRange.end)
    val quantileFeatures = DAL
      .readMostRecentSnapshot(AgathaQuantileFeaturesScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .flatMap { uqp =>
        uqp.quantiles.collect {
          case (quantile, score) if keepQuantiles.isEmpty || keepQuantiles.contains(quantile) =>
            ((uqp.featureClass, uqp.label, quantile), (uqp.userId, score))
        }
      }
      .forceToDisk

    val cutoffFeatureMapExec = createCutoffFeatureMapExec(quantileFeatures, defaultCutoffs)

    cutoffFeatureMapExec.flatMap { cutoffFeatures =>
      val filteredUsers = quantileFeatures
        .groupBy(_._2._1)
        .join(GetActiveUsers(DateRange(dateRange.start, RichDate.now)).asKeys)
        .withDescription("Filter to active users for score calculation")
        .mapValues { case ((key, userScore), _) => (key, userScore) }
        .values
        .forceToDisk

      val featureCutoffs = createPerFeatureThresholdMap(filteredUsers, cutoffFeatures)

      val featureScores =
        determinePerFeatureCutoffScores(filteredUsers, featureCutoffs, roundingPrecision)

      val records = quantileFeatures
        .join(featureScores)
        .withDescription("Apply appropriate rounded scores to original users")
        .flatMapValues {
          case ((userId, score), (minScore, feature)) =>
            if (score >= minScore)
              Some((userId, feature))
            else
              None
        }
        .values
        .group
        .withDescription("Create data records.")
        .mapGroup {
          case (userId, iter) =>
            val record = new DataRecord()
            record.setFeatureValue(SharedFeatures.USER_ID, userId)
            iter.foreach(binary => record.setFeatureValue(new Binary(binary), true))
            Iterator(record)
        }
        .values

      val binaryFeatures = cutoffFeatures.values.map(_.values).reduce(_ ++ _).toSeq
      val featureContext = baseFeatureContext.addFeatures(binaryFeatures: _*)

      DataSetPipe(records, featureContext)
        .writeDALSnapshotExecution(
          QuantileFeaturesDataRecordsJavaDataset,
          D.Daily,
          D.Suffix("predictions/userFeatureClass/quantileFeatureDataRecords"),
          D.EBLzo(),
          dateRange.end
        )
    }
  }
}

object QuantileFeatureExportJob extends TwitterExecutionApp {
  override def job: Execution[Unit] = {
    Execution.getArgs.flatMap { args =>
      implicit val tz: TimeZone = DateOps.UTC
      implicit val dp: DateParser = DateParser.default
      val dateRange = DateRange.parse(args.list("date"))

      BaseQuantileFeatureExport(dateRange)
    }
  }
}

object ScheduledQuantileFeatureExportJob extends TwitterScheduledExecutionApp {
  override def scheduledJob: Execution[Unit] = {
    Execution.getConfig.flatMap { execConfig =>
      val userDescription = execConfig.getArgs.required("batchDesc")
      val className = execConfig.getCascadingAppName.getOrElse(getClass.getCanonicalName)
      val batchDesc = s"$className:$userDescription"
      val args = execConfig.getArgs + (("batchDesc", Iterable(batchDesc)))

      AnalyticsBatchExecution(MediaScienceBatch.batchArgs(args)) { stateBirdDateRange: DateRange =>
        BaseQuantileFeatureExport(stateBirdDateRange)
      }
    }
  }
}
