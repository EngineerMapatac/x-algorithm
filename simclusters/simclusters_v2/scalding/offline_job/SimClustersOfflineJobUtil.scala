package com.twitter.simclusters_v2.scalding.offline_job

import com.twitter.algebird.DecayedValueMonoid
import com.twitter.algebird.Monoid
import com.twitter.algebird.OptionMonoid
import com.twitter.algebird_internal.thriftscala.{DecayedValue => ThriftDecayedValue}
import com.twitter.scalding.TypedPipe
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.common.Timestamp
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.hdfs_sources._
import com.twitter.simclusters_v2.summingbird.common.Configs
import com.twitter.simclusters_v2.summingbird.common.ThriftDecayedValueMonoid
import com.twitter.simclusters_v2.thriftscala._
import com.twitter.timelineservice.thriftscala.ContextualizedFavoriteEvent
import com.twitter.timelineservice.thriftscala.FavoriteEventUnion
import java.util.TimeZone
import twadoop_config.configuration.log_categories.group.timeline.TimelineServiceFavoritesScalaDataset

object SimClustersOfflineJobUtil {

  implicit val timeZone: TimeZone = DateOps.UTC
  implicit val dateParser: DateParser = DateParser.default

  implicit val modelVersionOrdering: Ordering[PersistedModelVersion] =
    Ordering.by(_.value)

  implicit val scoreTypeOrdering: Ordering[PersistedScoreType] =
    Ordering.by(_.value)

  implicit val persistedScoresOrdering: Ordering[PersistedScores] = Ordering.by(
    _.score.map(_.value).getOrElse(0.0)
  )

  implicit val decayedValueMonoid: DecayedValueMonoid = DecayedValueMonoid(0.0)

  implicit val thriftDecayedValueMonoid: ThriftDecayedValueMonoid =
    new ThriftDecayedValueMonoid(Configs.HalfLifeInMs)(decayedValueMonoid)

  implicit val persistedScoresMonoid: PersistedScoresMonoid =
    new PersistedScoresMonoid()(thriftDecayedValueMonoid)

  def readInterestedInScalaDataset(
    implicit dateRange: DateRange
  ): TypedPipe[(Long, ClustersUserIsInterestedIn)] = {
    DAL
      .readMostRecentSnapshot(
        SimclustersV2InterestedIn20M145K2020ScalaDataset,
        dateRange.embiggen(Months(6))
      )
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .map {
        case KeyVal(key, value) => (key, value)
      }
  }

  def readTimelineFavoriteData(
    implicit dateRange: DateRange
  ): TypedPipe[(UserId, TweetId, Timestamp)] = {
    DAL
      .read(TimelineServiceFavoritesScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .flatMap { cfe: ContextualizedFavoriteEvent =>
        cfe.event match {
          case FavoriteEventUnion.Favorite(fav) =>
            Some((fav.userId, fav.tweetId, fav.eventTimeMs))
          case _ =>
            None
        }

      }
  }

  class PersistedScoresMonoid(
    implicit thriftDecayedValueMonoid: ThriftDecayedValueMonoid)
      extends Monoid[PersistedScores] {

    private val optionalThriftDecayedValueMonoid =
      new OptionMonoid[ThriftDecayedValue]()

    override val zero: PersistedScores = PersistedScores()

    override def plus(x: PersistedScores, y: PersistedScores): PersistedScores = {
      PersistedScores(
        optionalThriftDecayedValueMonoid.plus(
          x.score,
          y.score
        )
      )
    }

    def build(value: Double, timeInMs: Double): PersistedScores = {
      PersistedScores(Some(thriftDecayedValueMonoid.build(value, timeInMs)))
    }
  }

}
