package com.twitter.agatha.scalding.util

import java.util.TimeZone

import com.twitter.healthde.suspension_events.SuspensionEventsScalaDataset
import com.twitter.scalding.serialization.RequiredBinaryComparators._
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access._

object GetSuspendedUsers {
  def apply(dateRange: DateRange)(implicit tz: TimeZone): TypedPipe[Long] = {
    val reasons = Set("other", "spam")
    val chaSuspensionReason = "#chaq"
    DAL
      .read(SuspensionEventsScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .collect {
        case event
            if event.action.toLowerCase == "suspend" && reasons.contains(
              event.reason.toLowerCase()) &&
              !event.note.exists(_.toLowerCase.contains(chaSuspensionReason)) =>
          event.userId
      }.distinct
  }
}
