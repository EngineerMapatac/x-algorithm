package com.twitter.agatha.scalding.util

import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.usersource.snapshot.combined.UsersourceScalaDataset

object GetActiveUsers {
  def apply(dateRange: DateRange): TypedPipe[Long] = {
    DAL
      .readMostRecentSnapshot(UsersourceScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .flatMap { combinedUser =>
        for {
          user <- combinedUser.user
          safety <- user.safety
          if !(safety.deactivated || safety.suspended)
          extendedUser <- combinedUser.userExtended
          userState <- extendedUser.userState
          if userState.value != 1
        } yield user.id
      }
  }
}
