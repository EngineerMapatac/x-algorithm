package com.twitter.agatha.scalding.data.loaders

import com.twitter.dal.client.dataset.SnapshotDALDataset
import com.twitter.data.proto.Flock
import com.twitter.hub.agatha.thriftscala.FeatureSource
import graphstore.common.FlockFollowsJavaDataset

case class FollowGraphDataLoader(createReverseFeatures: Boolean)
    extends MostRecentEdgeSourceDataLoader {
  override val flockDataset: SnapshotDALDataset[Flock.Edge] = FlockFollowsJavaDataset
  override val identifier: FeatureSource = FeatureSource.FollowGraph
}
