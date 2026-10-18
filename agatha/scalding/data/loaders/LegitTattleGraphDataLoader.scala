package com.twitter.agatha.scalding.data.loaders

import com.twitter.agatha.scalding.data.DataLoaderUtil
import com.twitter.agatha.scalding.data.GraphBasedDataLoader
import com.twitter.hub.agatha.thriftscala.Feature
import com.twitter.hub.agatha.thriftscala.FeatureSource
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.serialization.RequiredBinaryComparators._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.spam.scalding.datasets.TattleEventsScalaDataset

case class LegitTattleGraphDataLoader(createReverseFeatures: Boolean) extends GraphBasedDataLoader {
  import DataLoaderUtil._

  override def read(dateRange: DateRange): TypedPipe[(Long, Feature)] = {
    DAL
      .read(TattleEventsScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .collect {
        case tattle
            if notTestAccount(tattle.spammerId) &&
              notTestAccount(tattle.victimId) && tattle.isLegit.getOrElse(false) =>
          (tattle.spammerId, tattle.victimId)
      }.distinct
      .flatMap {
        case (spammerId, victimId) =>
          getDirectionalGraphFeaturesFromUserPair(
            spammerId,
            victimId,
            FeatureSource.LegitTattleGraph,
            createReverseFeatures
          )
      }
  }
}
