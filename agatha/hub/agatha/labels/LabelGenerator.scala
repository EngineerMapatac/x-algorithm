package com.twitter.hub.agatha.labels

import com.twitter.hub.agatha.thriftscala.UserLabel
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe

trait LabelGenerator {
  val labelName: String
  def read(dateRange: DateRange): TypedPipe[UserLabel]
}
