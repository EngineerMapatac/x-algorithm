package com.twitter.agatha.scalding

import com.twitter.agatha.scalding.data.ActiveUserFeaturesScalaDataset
import com.twitter.agatha.scalding.labels.AgathaLabelManager
import com.twitter.agatha.scalding.labels.nsfw.NSFWLabel
import com.twitter.agatha.scalding.labels.reports_per_fav.ReportsPerFav
import com.twitter.agatha.scalding.labels.spam_reports_per_fav.AllSpamReportsPerFav
import com.twitter.agatha.scalding.labels.spam_reports_per_fav.SpamReportsPerFav
import com.twitter.agatha.scalding.labels.spam_suspended.SpamSuspendedLabel
import com.twitter.agatha.thriftscala.UserPrediction
import com.twitter.hub.agatha.job.PredictJobConfig
import com.twitter.hub.agatha.job.PredictJobTemplate
import com.twitter.hub.agatha.job.ScheduledPredictJobTemplate
import com.twitter.hub.agatha.labels.LabelManager
import com.twitter.hub.agatha.predict.Aggregators
import com.twitter.hub.agatha.predict.UserGraphGrouping
import com.twitter.hub.agatha.thriftscala._
import com.twitter.hub.util.metadata_source.MetadataSource
import com.twitter.scalding._
import com.twitter.scalding.typed.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla

object AgathaFeaturesJobConfig extends PredictJobConfig {
  override val historyDuration: Duration = Days(14)
  override val labelManager: LabelManager = AgathaLabelManager
  override val labelNames: Set[String] = Set(
    ReportsPerFav,
    AllSpamReportsPerFav,
    SpamReportsPerFav,
    SpamSuspendedLabel,
    NSFWLabel,
  ).map(_.labelName)
  val quantiles: Seq[Double] = Seq(.1, .25, .5, .75, .9)

  override def readUserFeatures(dateRange: DateRange): TypedPipe[UserFeature] = {
    DAL
      .readMostRecentSnapshot(ActiveUserFeaturesScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
  }

  override def readModelledFeatures(
    dateRange: DateRange
  )(
    implicit mode: Mode
  ): Map[TypedPipe[ModelledFeature], ModelledFeatureLabelMetadata] = {
    Seq(
      RateBasedModelledFeatureScalaDataset,
      SpagathaModelledFeatureScalaDataset,
      NsfwModelledFeatureScalaDataset,
    ).map { source =>
      val expandedDateRange = DateRange(dateRange.start - Days(14)(DateOps.UTC), dateRange.end)
      MetadataSource.read(
        DAL
          .readMostRecentSnapshot(source, expandedDateRange)
          .withRemoteReadPolicy(ExplicitLocation(ProcAtla)),
        ModelledFeatureLabelMetadata
      )
    }.toMap
  }

  override def outputPredictions(
    pmiSet: TypedPipe[((UserGraphGrouping, String), Double)],
    dateRange: DateRange
  ): Execution[Unit] = {
    Aggregators
      .getPresentMoments(pmiSet)
      .map {
        case (userId, featureClass, label, moments) =>
          UserPrediction(userId, featureClass, label, moments)
      }
      .map(identity)
      .writeDALSnapshotExecution(
        AgathaFeaturesScalaDataset,
        D.Daily,
        D.Suffix("predictions/userFeatureClass/agathaFeatures"),
        D.Parquet,
        dateRange.end
      )
  }
}

object AgathaFeaturesPredictJob extends PredictJobTemplate(AgathaFeaturesJobConfig)
object ScheduledAgathaFeaturesPredictJob
    extends ScheduledPredictJobTemplate(AgathaFeaturesJobConfig)
