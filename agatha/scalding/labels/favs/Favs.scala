package com.twitter.agatha.scalding.labels.favs

import com.twitter.agatha.scalding.data.DataLoaderUtil
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.serialization.RequiredBinaryComparators._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.timelineservice.thriftscala.FavoriteEventUnion
import graphstore.common.FlockFollowsJavaDataset
import twadoop_config.configuration.log_categories.group.timeline.TimelineServiceFavoritesScalaDataset

object Favs {
  def readOutOfNetworkFavs(dateRange: DateRange): TypedPipe[(Long, Long)] = {
    val favs = getFavs(dateRange)
    val inNetwork = getInNetwork(dateRange)
    getOutOfNetworkFavs(favs, inNetwork)
  }

  def getFavs(implicit dateRange: DateRange): TypedPipe[(Long, Long)] = {
    DAL
      .read(TimelineServiceFavoritesScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .flatMap { favEvent =>
        favEvent.event match {
          case FavoriteEventUnion.Favorite(favorite)
              if favorite.userId != favorite.tweetUserId &&
                DataLoaderUtil.notTestAccount(favorite.tweetUserId) &&
                DataLoaderUtil.notTestAccount(favorite.userId) =>
            Some((favorite.tweetUserId, favorite.userId))
          case _ => None
        }
      }
      .distinct
  }

  def getOutOfNetworkFavs(
    favs: TypedPipe[(Long, Long)],
    inNetwork: TypedPipe[(Long, Long)]
  ): TypedPipe[(Long, Long)] = {
    favs.asKeys
      .leftJoin(inNetwork.asKeys)
      .collect {
        case ((dest, source), (_, joined)) if joined.isEmpty =>
          (dest, source)
      }
  }

  def getInNetwork(implicit dateRange: DateRange): TypedPipe[(Long, Long)] = {
    DAL
      .readMostRecentSnapshot(
        FlockFollowsJavaDataset,
        dateRange
      )
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .flatMap { edge =>
        val (sourceId, destId) = (edge.getSourceId, edge.getDestinationId)
        Seq((sourceId, destId), (destId, sourceId))
      }
      .forceToDisk
      .asKeys
      .sum
      .withReducers(5000)
      .keys
  }
}
