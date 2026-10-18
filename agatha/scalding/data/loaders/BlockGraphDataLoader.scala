package com.twitter.agatha.scalding.data.loaders

import com.twitter.dal.client.dataset.SnapshotDALDataset
import com.twitter.data.proto.Flock
import com.twitter.hub.agatha.thriftscala.FeatureSource
import graphstore.common.FlockBlocksJavaDataset

case class BlockGraphDataLoader(createReverseFeatures: Boolean)
    extends MostRecentEdgeSourceDataLoader {
  override val flockDataset: SnapshotDALDataset[Flock.Edge] = FlockBlocksJavaDataset
  override val identifier: FeatureSource = FeatureSource.BlockGraph
}
