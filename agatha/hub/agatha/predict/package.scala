package com.twitter.hub.agatha

import com.twitter.hub.agatha.thriftscala.FeatureClass

package object predict {
  type UserGraphGrouping = (Long, FeatureClass)
}
