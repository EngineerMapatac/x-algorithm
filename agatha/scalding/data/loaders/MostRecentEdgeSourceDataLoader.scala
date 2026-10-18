package com.twitter.agatha.scalding.data.loaders

import com.twitter.agatha.scalding.data.DataLoaderUtil
import com.twitter.agatha.scalding.data.GraphBasedDataLoader
import com.twitter.dal.client.dataset.SnapshotDALDataset
import com.twitter.data.proto.Flock
import com.twitter.hub.agatha.thriftscala.Feature
import com.twitter.hub.agatha.thriftscala.FeatureSource
import com.twitter.scalding.DateOps
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla

trait MostRecentEdgeSourceDataLoader extends GraphBasedDataLoader {
  import DataLoaderUtil._

  val flockDataset: SnapshotDALDataset[Flock.Edge]

  val identifier: FeatureSource

  override def read(dateRange: DateRange): TypedPipe[(Long, Feature)] = {
    DAL
      .readMostRecentSnapshot(
        flockDataset,
        DateRange(dateRange.start - Days(7)(DateOps.UTC), dateRange.end)
      )
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .withDescription(s"EdgeDataLoader: ${identifier.name}")
      .collect {
        case edge
            if Set(0, 2).contains(edge.getStateId) &&
              notTestAccount(edge.getSourceId) &&
              notTestAccount(edge.getDestinationId) =>
          (edge.getSourceId, edge.getDestinationId)
      }
      .flatMap {
        case (sourceId, destId) =>
          getDirectionalGraphFeaturesFromUserPair(
            sourceId,
            destId,
            identifier,
            createReverseFeatures)
      }
  }
}
