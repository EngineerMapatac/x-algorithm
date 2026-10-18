package com.twitter.agatha.scalding.data.loaders

import com.twitter.dal.client.dataset.SnapshotDALDataset
import com.twitter.data.proto.Flock
import com.twitter.hub.agatha.thriftscala.FeatureSource
import graphstore.common.FlockContributorsJavaDataset

case class ContributorGraphDataLoader(createReverseFeatures: Boolean)
    extends MostRecentEdgeSourceDataLoader {
  override val flockDataset: SnapshotDALDataset[Flock.Edge] = FlockContributorsJavaDataset
  override val identifier: FeatureSource = FeatureSource.ContributorGraph
}
