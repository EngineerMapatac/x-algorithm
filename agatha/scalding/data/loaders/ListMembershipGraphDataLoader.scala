package com.twitter.agatha.scalding.data.loaders

import com.twitter.dal.client.dataset.SnapshotDALDataset
import com.twitter.data.proto.Flock
import com.twitter.hub.agatha.thriftscala.FeatureSource
import graphstore.common.FlockBelongsToGroupJavaDataset

object ListMembershipGraphDataLoader extends MostRecentEdgeSourceDataLoader {
  override val flockDataset: SnapshotDALDataset[Flock.Edge] = FlockBelongsToGroupJavaDataset
  override val identifier: FeatureSource = FeatureSource.ListMembership

  override val createReverseFeatures: Boolean = false
}
