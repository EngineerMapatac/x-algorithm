package com.twitter.agatha.scalding.data.loaders

import com.twitter.dal.client.dataset.SnapshotDALDataset
import com.twitter.data.proto.Flock
import com.twitter.hub.agatha.thriftscala.FeatureSource
import graphstore.common.FlockSubscribesToGroupJavaDataset

object ListFollowerGraphDataLoader extends MostRecentEdgeSourceDataLoader {
  override val flockDataset: SnapshotDALDataset[Flock.Edge] = FlockSubscribesToGroupJavaDataset
  override val identifier: FeatureSource = FeatureSource.ListFollowers

  override val createReverseFeatures: Boolean = false
}
