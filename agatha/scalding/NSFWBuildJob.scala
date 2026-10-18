package com.twitter.agatha.scalding

import com.twitter.agatha.scalding.data.ActiveUserFeaturesScalaDataset
import com.twitter.agatha.scalding.labels.nsfw.NSFWLabel
import com.twitter.agatha.scalding.labels.nsfw.NsfwAgathaLabelManager
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

object NSFWBuildJobConfig extends BuildJobConfig {
  override val historyDuration: Duration = Days(30)
  override val labelManager: LabelManager = NsfwAgathaLabelManager
  override val labelNames: Set[String] = Set(
    NSFWLabel.labelName,
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
      NsfwModelledFeatureScalaDataset,
      D.Daily,
      D.Suffix("models/nsfw"),
      D.Parquet,
      dateRange.end
    )
  }
}

object NSFWBuildJob
    extends BuildJobTemplate(NSFWBuildJobConfig)
    with RequiredBinaryComparatorsExecutionApp

object ScheduledNSFWBuildJob extends ScheduledBuildJobTemplate(NSFWBuildJobConfig)
