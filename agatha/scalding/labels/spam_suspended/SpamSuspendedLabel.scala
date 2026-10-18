package com.twitter.agatha.scalding.labels.spam_suspended

import com.twitter.hub.agatha.labels.LabelGenerator
import com.twitter.hub.agatha.thriftscala.UserLabel
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla

object SpamSuspendedLabel extends LabelGenerator {
  override val labelName: String = "SpamSuspended"

  override def read(dateRange: DateRange): TypedPipe[UserLabel] = {
    DAL
      .readMostRecentSnapshot(SpamSuspendedUserLabelScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
  }
}
