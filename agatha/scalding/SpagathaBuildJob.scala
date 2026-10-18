package com.twitter.agatha.scalding

import com.twitter.agatha.scalding.data.ActiveUserFeaturesScalaDataset
import com.twitter.agatha.scalding.labels.AgathaLabelManager
import com.twitter.agatha.scalding.labels.spam_suspended.SpamSuspendedLabel
import com.twitter.hub.agatha.job.BuildJobConfig
import com.twitter.hub.agatha.job.BuildJobTemplate
import com.twitter.hub.agatha.job.ScheduledBuildJobTemplate
import com.twitter.hub.agatha.labels.LabelManager
import com.twitter.hub.agatha.thriftscala._
import com.twitter.hub.util.metadata_source.MetadataSource
import com.twitter.scalding._
import com.twitter.scalding.serialization.RequiredBinaryComparatorsExecutionApp
import com.twitter.scalding.typed.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite.D
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla

object SpagathaJobConfig extends BuildJobConfig {
  override val historyDuration: Duration = Days(30)
  override val labelManager: LabelManager = AgathaLabelManager
  override val labelNames: Set[String] = Set(
    SpamSuspendedLabel.labelName,
  )

  override def readUserFeatures(dateRange: DateRange): TypedPipe[UserFeature] = {
    DAL
      .readMostRecentSnapshot(ActiveUserFeaturesScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
  }

  override def writeModelledFeatures(
    modelledFeature: TypedPipe[ModelledFeature],
    metadata: ModelledFeatureLabelMetadata,
    dateRange: DateRange
  ): Execution[Unit] = {
    new MetadataSource(
      modelledFeature.forceToDisk.map(identity),
      metadata,
      ModelledFeatureLabelMetadata
    ).writeDALSnapshotExecution(
      SpagathaModelledFeatureScalaDataset,
      D.Daily,
      D.Suffix("models/spagatha"),
      D.Parquet,
      dateRange.end
    )
  }
}

object SpagathaBuildJob
    extends BuildJobTemplate(SpagathaJobConfig)
    with RequiredBinaryComparatorsExecutionApp
object ScheduledSpagathaBuildJob extends ScheduledBuildJobTemplate(SpagathaJobConfig)
