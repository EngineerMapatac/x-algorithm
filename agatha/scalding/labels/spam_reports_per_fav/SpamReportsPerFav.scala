package com.twitter.agatha.scalding.labels.spam_reports_per_fav

import com.twitter.hub.agatha.thriftscala.UserLabel
import com.twitter.hub.agatha.labels.LabelGenerator
import com.twitter.scalding._
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import java.util.TimeZone

object SpamReportsPerFav extends LabelGenerator {
  override val labelName: String = "SpamReportsPerFav"
  implicit val tz: TimeZone = DateOps.UTC

  override def read(dateRange: DateRange): TypedPipe[UserLabel] = {
    DAL
      .readMostRecentSnapshot(SpamReportsPerFavUserLabelScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
  }
}

object AllSpamReportsPerFav extends LabelGenerator {
  override val labelName: String = "AllSpamReportsPerFav"
  implicit val tz: TimeZone = DateOps.UTC

  override def read(dateRange: DateRange): TypedPipe[UserLabel] = {
    DAL
      .readMostRecentSnapshot(AllSpamReportsPerFavUserLabelScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
  }
}
