package com.twitter.agatha.scalding.labels.spam_suspended

import com.twitter.agatha.scalding.labels.LabelExtractionConfig
import com.twitter.agatha.scalding.labels.LabelExtractionTemplate
import com.twitter.agatha.scalding.labels.LabelUtil
import com.twitter.agatha.scalding.labels.LabelledUsers
import com.twitter.agatha.scalding.labels.ScheduledLabelExtractionTemplate
import com.twitter.agatha.scalding.util.GetSuspendedUsers
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.Duration
import com.twitter.scalding.Execution
import com.twitter.scalding.serialization.RequiredBinaryComparatorsExecutionApp

object SpamSuspendedLabelExtraction extends LabelExtractionConfig {
  override val historyDuration: Duration = Days(30)

  override def generateLabels(dateRange: DateRange): Execution[Seq[LabelledUsers]] = {
    val userLabels = LabelUtil.createPositiveNegativeLabelsFromActiveUsers(
      GetSuspendedUsers(dateRange),
      SpamSuspendedLabel.labelName,
      dateRange)
    Execution.from(
      Seq(
        LabelledUsers(userLabels, SpamSuspendedLabel.labelName, SpamSuspendedUserLabelScalaDataset)
      ))
  }
}

object SpamSuspendedExtractionJob
    extends LabelExtractionTemplate(SpamSuspendedLabelExtraction)
    with RequiredBinaryComparatorsExecutionApp
object ScheduledSpamSuspendedExtractionJob
    extends ScheduledLabelExtractionTemplate(SpamSuspendedLabelExtraction)
    with RequiredBinaryComparatorsExecutionApp
