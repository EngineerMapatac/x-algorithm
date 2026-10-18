package com.twitter.agatha.scalding.labels.nsfw

import com.twitter.agatha.scalding.labels._
import com.twitter.hub.agatha.thriftscala.UserLabel
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.scalding_internal.job.RequiredBinaryComparatorsExecutionApp
import com.twitter.visibility.user_labels.UserLabelRecordsScalaDataset

object NSFWLabelExtraction extends LabelExtractionConfig {
  override val historyDuration: Duration = Days(30)

  def generatePositivesAndNegativeNSFWUsers(dateRange: DateRange): TypedPipe[UserLabel] = {
    val NSFWLabels: Set[String] = Set(
      "NsfwHighPrecision",
      "NsfwAvatarImage",
      "NsfwBannerImage",
    )
    val positives = DAL
      .readMostRecentSnapshot(
        UserLabelRecordsScalaDataset,
        dateRange
      )
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .collect { case ul if NSFWLabels.contains(ul.labelValue) => ul.userId }

    LabelUtil.createPositiveNegativeLabelsFromActiveUsers(positives, NSFWLabel.labelName, dateRange)
  }

  override def generateLabels(dateRange: DateRange): Execution[Seq[LabelledUsers]] = {
    Execution.from(
      Seq(
        LabelledUsers(
          generatePositivesAndNegativeNSFWUsers(dateRange),
          NSFWLabel.labelName,
          NsfwUserLabelScalaDataset
        )
      ))
  }
}

object NSFWExtractionJob
    extends LabelExtractionTemplate(NSFWLabelExtraction)
    with RequiredBinaryComparatorsExecutionApp
object ScheduledNSFWExtractionJob
    extends ScheduledLabelExtractionTemplate(NSFWLabelExtraction)
    with RequiredBinaryComparatorsExecutionApp
