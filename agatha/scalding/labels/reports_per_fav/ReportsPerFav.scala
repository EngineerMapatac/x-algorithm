package com.twitter.agatha.scalding.labels.reports_per_fav

import com.twitter.hub.agatha.labels.LabelGenerator
import com.twitter.hub.agatha.thriftscala.UserLabel
import com.twitter.scalding._
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import java.util.TimeZone

object ReportsPerFav extends LabelGenerator {
  override val labelName: String = "ReportsPerFav"
  implicit val tz: TimeZone = DateOps.UTC

  override def read(dateRange: DateRange): TypedPipe[UserLabel] = {
    DAL
      .readMostRecentSnapshot(ReportsPerFavUserLabelScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
  }
}

object MentionFilteredReportsPerFav extends LabelGenerator {
  override val labelName: String = "MentionFilteredReportsPerFav"
  implicit val tz: TimeZone = DateOps.UTC

  override def read(dateRange: DateRange): TypedPipe[UserLabel] = {
    DAL
      .readMostRecentSnapshot(
        MentionFilteredReportsPerFavUserLabelScalaDataset,
        dateRange
      )
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
  }
}
