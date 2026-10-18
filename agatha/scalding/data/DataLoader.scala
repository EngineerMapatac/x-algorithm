package com.twitter.agatha.scalding.data

import com.twitter.hub.agatha.thriftscala.Feature
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.serialization.RequiredBinaryComparators._

trait DataLoader {

  def read(dateRange: DateRange): TypedPipe[(Long, Feature)]
}

trait GraphBasedDataLoader extends DataLoader {
  val createReverseFeatures: Boolean
}
