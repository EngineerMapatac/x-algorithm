package com.twitter.agatha.scalding.labels.nsfw

import com.twitter.hub.agatha.labels.LabelGenerator
import com.twitter.hub.agatha.thriftscala.UserLabel
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL

object NSFWLabel extends LabelGenerator {
  override val labelName: String = "NSFW"

  override def read(dateRange: DateRange): TypedPipe[UserLabel] = {
    DAL.readMostRecentSnapshot(NsfwUserLabelScalaDataset, dateRange).toTypedPipe
  }
}
