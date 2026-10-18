package com.twitter.agatha.scalding.labels.rate_based_labels

import com.twitter.agatha.scalding.labels.LabelExtractionConfig
import com.twitter.agatha.scalding.labels.LabelExtractionTemplate
import com.twitter.agatha.scalding.labels.LabelledUsers
import com.twitter.agatha.scalding.labels.ScheduledLabelExtractionTemplate
import com.twitter.scalding._
import com.twitter.scalding_internal.job.RequiredBinaryComparatorsExecutionApp

object RateBasedLabelsExtraction extends LabelExtractionConfig {
  override val historyDuration: Duration = Days(30)

  override def generateLabels(dateRange: DateRange): Execution[Seq[LabelledUsers]] = {
    RateBasedLabels.getUserLabels(dateRange)
  }
}

object RateBasedExtractionJob
    extends LabelExtractionTemplate(RateBasedLabelsExtraction)
    with RequiredBinaryComparatorsExecutionApp
object ScheduledRateBasedExtractionJob
    extends ScheduledLabelExtractionTemplate(RateBasedLabelsExtraction)
    with RequiredBinaryComparatorsExecutionApp
