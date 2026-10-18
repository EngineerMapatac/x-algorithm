package com.twitter.agatha.scalding.data.loaders

import com.twitter.agatha.scalding.data.DataLoaderUtil
import com.twitter.agatha.scalding.data.GraphBasedDataLoader
import com.twitter.health_seh.jobs.dm_participants.DmParticipantsScalaDataset
import com.twitter.health_seh.jobs.dm_participants.thriftscala.DmParticipants
import com.twitter.hub.agatha.thriftscala.Feature
import com.twitter.hub.agatha.thriftscala.FeatureSource
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.AllowCrossClusterSameDC
import com.twitter.scalding_internal.job.RequiredBinaryComparators.ordSer

case class DirectMessageGraphDataLoader(createReverseFeatures: Boolean)
    extends GraphBasedDataLoader {
  import DataLoaderUtil._

  override def read(dateRange: DateRange): TypedPipe[(Long, Feature)] = {
    DAL
      .read(DmParticipantsScalaDataset, dateRange)
      .withRemoteReadPolicy(AllowCrossClusterSameDC)
      .toTypedPipe
      .flatMap {
        case DmParticipants(_, senderId, receiverId) =>
          getDirectionalGraphFeaturesFromUserPair(
            senderId,
            receiverId,
            FeatureSource.DirectMessageGraph,
            createReverseFeatures
          )
      }.distinct
  }
}
