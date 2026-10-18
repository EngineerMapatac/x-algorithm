package com.twitter.simclusters_v2.scalding.tweet_similarity

import com.twitter.ads.entities.db.thriftscala.PromotedTweet
import com.twitter.dataproducts.estimation.ReservoirSampler
import com.twitter.scalding.typed.TypedPipe
import com.twitter.scalding.DateRange
import com.twitter.scalding.Execution
import com.twitter.scalding.TypedTsv
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.Proc3Atla
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.simclusters_v2.common.SimClustersEmbedding
import com.twitter.simclusters_v2.common.Timestamp
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.scalding.common.Util
import com.twitter.simclusters_v2.scalding.embedding.common.ExternalDataSources
import com.twitter.simclusters_v2.thriftscala.TweetTopKTweetsWithScore
import com.twitter.simclusters_v2.thriftscala.TweetWithScore
import com.twitter.simclusters_v2.thriftscala.TweetsWithScore
import com.twitter.timelineservice.thriftscala.ContextualizedFavoriteEvent
import com.twitter.timelineservice.thriftscala.FavoriteEventUnion
import com.twitter.wtf.scalding.client_event_processing.thriftscala.InteractionDetails
import com.twitter.wtf.scalding.client_event_processing.thriftscala.InteractionType
import com.twitter.wtf.scalding.client_event_processing.thriftscala.TweetImpressionDetails
import com.twitter.wtf.scalding.jobs.client_event_processing.UserInteractionScalaDataset
import java.util.Random
import scala.collection.mutable.ArrayBuffer
import scala.util.control.Breaks._
import twadoop_config.configuration.log_categories.group.timeline.TimelineServiceFavoritesScalaDataset

object TweetPairLabelCollectionUtil {

  case class FeaturedTweet(
    tweet: TweetId,
    timestamp: Timestamp,
    author: Option[UserId],
    embedding: Option[SimClustersEmbedding])
      extends Ordered[FeaturedTweet] {

    import scala.math.Ordered.orderingToOrdered

    def compare(that: FeaturedTweet): Int =
      (this.tweet, this.timestamp, this.author) compare (that.tweet, that.timestamp, that.author)
  }

  val MaxFavPerUser: Int = 100

  def getFavEvents(
    dateRange: DateRange,
    maxOutgoingDegree: Int
  ): TypedPipe[(UserId, TweetId, Timestamp)] = {
    val fullTimelineFavData: TypedPipe[ContextualizedFavoriteEvent] =
      DAL
        .read(TimelineServiceFavoritesScalaDataset, dateRange)
        .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
        .toTypedPipe

    val userTweetTuples = fullTimelineFavData
      .flatMap { cfe: ContextualizedFavoriteEvent =>
        cfe.event match {
          case FavoriteEventUnion.Favorite(fav) =>
            Some((fav.userId, (fav.tweetId, fav.eventTimeMs)))
          case _ =>
            None
        }
      }
    val usersWithValidOutDegree = userTweetTuples
      .groupBy(_._1)
      .withReducers(1000)
      .size
      .filter(_._2 <= maxOutgoingDegree)

    userTweetTuples
      .join(usersWithValidOutDegree).map {
        case (userId, ((tweetId, eventTime), _)) => (userId, tweetId, eventTime)
      }.forceToDisk
  }

  def getImpressionEvents(dateRange: DateRange): TypedPipe[(UserId, TweetId, Timestamp)] = {
    DAL
      .read(UserInteractionScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(Proc3Atla))
      .toTypedPipe
      .flatMap {
        case userInteraction
            if userInteraction.interactionType == InteractionType.TweetImpressions =>
          userInteraction.interactionDetails match {
            case InteractionDetails.TweetImpressionDetails(
                  TweetImpressionDetails(tweetId, _, dwellTimeInSecOpt))
                if dwellTimeInSecOpt.exists(_ >= 1) =>
              Some(userInteraction.userId, tweetId, userInteraction.timeStamp)
            case _ =>
              None
          }
        case _ => None
      }
      .forceToDisk
  }

  def getFilteredEvents(
    events: TypedPipe[(UserId, TweetId, Timestamp)],
    tweets: TypedPipe[TweetId]
  ): TypedPipe[(UserId, TweetId, Timestamp)] = {
    events
      .map {
        case (userId, tweetId, eventTime) => (tweetId, (userId, eventTime))
      }
      .join(tweets.asKeys)
      .withReducers(1000)
      .map {
        case (tweetId, ((userId, eventTime), _)) => (userId, tweetId, eventTime)
      }
  }

  def getTweetAuthorPairs(dateRange: DateRange): TypedPipe[(TweetId, UserId)] = {
    ExternalDataSources
      .flatTweetsSource(dateRange)
      .collect {
        case record if record.shareSourceTweetId.isEmpty && record.quotedTweetTweetId.isEmpty =>
          (record.tweetId, record.userId)
      }
  }

  def getNonPromotedTweets(
    promotedTweets: TypedPipe[PromotedTweet],
    tweets: TypedPipe[TweetId]
  ): TypedPipe[TweetId] = {
    promotedTweets
      .collect {
        case promotedTweet if promotedTweet.tweetId.isDefined => promotedTweet.tweetId.get
      }
      .asKeys
      .rightJoin(tweets.asKeys)
      .withReducers(1000)
      .filterNot(joined => joined._2._1.isDefined)
      .keys
  }

  def getTweetPairs(
    events: TypedPipe[(UserId, FeaturedTweet)],
    timeframe: Long,
    isCoengaged: Boolean
  ): TypedPipe[(UserId, FeaturedTweet, FeaturedTweet, Boolean)] = {
    events
      .map {
        case (userId, featuredTweet) => (userId, Seq(featuredTweet))
      }
      .sumByKey
      .flatMap {
        case (userId, featuredTweets) if featuredTweets.size > 1 =>
          val sortedFeaturedTweet = featuredTweets.sortBy(_.timestamp)
          val distinctPairs = ArrayBuffer[(UserId, FeaturedTweet, FeaturedTweet, Boolean)]()
          breakable {
            for (i <- sortedFeaturedTweet.indices) {
              for (j <- i + 1 until sortedFeaturedTweet.size) {
                val featuredTweet1 = sortedFeaturedTweet(i)
                val featuredTweet2 = sortedFeaturedTweet(j)
                if (math.abs(featuredTweet1.timestamp - featuredTweet2.timestamp) <= timeframe)
                  distinctPairs ++= Seq(
                    (userId, featuredTweet1, featuredTweet2, isCoengaged),
                    (userId, featuredTweet2, featuredTweet1, isCoengaged))
                else
                  break
              }
            }
          }
          distinctPairs
        case _ => Nil
      }
  }

  def getCoengagedPairs(
    favEvents: TypedPipe[(UserId, TweetId, Timestamp)],
    tweets: TypedPipe[TweetId],
    coengagementTimeframe: Long
  ): TypedPipe[(UserId, FeaturedTweet, FeaturedTweet, Boolean)] = {
    val userFeaturedTweetPairs =
      getFilteredEvents(favEvents, tweets)
        .map {
          case (user, tweet, timestamp) => (user, FeaturedTweet(tweet, timestamp, None, None))
        }

    getTweetPairs(userFeaturedTweetPairs, coengagementTimeframe, isCoengaged = true)
  }

  def getCoimpressedPairs(
    impressionEvents: TypedPipe[(UserId, TweetId, Timestamp)],
    tweets: TypedPipe[TweetId],
    timeframe: Long
  ): TypedPipe[(UserId, FeaturedTweet, FeaturedTweet, Boolean)] = {
    val userFeaturedTweetPairs = getFilteredEvents(impressionEvents, tweets)
      .map {
        case (user, tweet, timestamp) => (user, FeaturedTweet(tweet, timestamp, None, None))
      }

    getTweetPairs(userFeaturedTweetPairs, timeframe, isCoengaged = false)
  }

  def computeLabelledTweetPairs(
    coengagedPairs: TypedPipe[(UserId, FeaturedTweet, FeaturedTweet, Boolean)],
    coimpressedPairs: TypedPipe[(UserId, FeaturedTweet, FeaturedTweet, Boolean)]
  ): TypedPipe[(FeaturedTweet, FeaturedTweet, Boolean)] = {
    (coengagedPairs ++ coimpressedPairs)
      .groupBy {
        case (userId, queryFeaturedTweet, candidateFeaturedTweet, _) =>
          (userId, queryFeaturedTweet.tweet, candidateFeaturedTweet.tweet)
      }
      .maxBy {
        case (_, _, _, label) => label
      }
      .values
      .map { case (_, queryTweet, candidateTweet, label) => (queryTweet, candidateTweet, label) }
  }

  def getQueryTweetBalancedClassPairs(
    labelledPairs: TypedPipe[(FeaturedTweet, FeaturedTweet, Boolean)],
    maxSamplesPerClass: Int
  ): TypedPipe[(FeaturedTweet, FeaturedTweet, Boolean)] = {
    val queryTweetToSampleCount = labelledPairs
      .map {
        case (queryTweet, _, label) =>
          if (label) (queryTweet.tweet, (1, 0)) else (queryTweet.tweet, (0, 1))
      }
      .sumByKey
      .map {
        case (queryTweet, (posCount, negCount)) =>
          (queryTweet, Math.min(Math.min(posCount, negCount), maxSamplesPerClass))
      }

    labelledPairs
      .groupBy { case (queryTweet, _, _) => queryTweet.tweet }
      .join(queryTweetToSampleCount)
      .values
      .map {
        case ((queryTweet, candidateTweet, label), samplePerClass) =>
          ((queryTweet.tweet, label, samplePerClass), (queryTweet, candidateTweet, label))
      }
      .group
      .mapGroup {
        case ((_, _, samplePerClass), iter) =>
          val random = new Random(123L)
          val sampler =
            new ReservoirSampler[(FeaturedTweet, FeaturedTweet, Boolean)](samplePerClass, random)
          iter.foreach { pair => sampler.sampleItem(pair) }
          sampler.sample.toIterator
      }
      .values
  }

  def getScoredCoengagedTweetPairs(
    events: TypedPipe[(UserId, TweetId, Timestamp)],
    minInDegree: Int,
    coengagementTimeframe: Long
  )(
  ): TypedPipe[(TweetId, TweetWithScore)] = {

    val tweetNorms = events
      .map { case (_, tweetId, _) => (tweetId, 1.0) }
      .sumByKey
      .filter(_._2 >= minInDegree)
      .mapValues(math.sqrt)

    val edgesWithWeight = events
      .map {
        case (userId, tweetId, eventTime) => (tweetId, (userId, eventTime))
      }
      .join(tweetNorms)
      .map {
        case (tweetId, ((userId, eventTime), norm)) =>
          (userId, Seq((tweetId, eventTime, 1 / norm)))
      }

    val tweetPairsWithWeight = edgesWithWeight.sumByKey
      .flatMap {
        case (_, tweets) if tweets.size > 1 =>
          allUniquePairs(tweets).flatMap {
            case ((tweetId1, eventTime1, weight1), (tweetId2, eventTime2, weight2)) =>
              if ((eventTime1 - eventTime2).abs <= coengagementTimeframe) {
                if (tweetId1 > tweetId2)
                  Some(((tweetId2, tweetId1), weight1 * weight2))
                else
                  Some(((tweetId1, tweetId2), weight1 * weight2))
              } else {
                None
              }
            case _ =>
              None
          }
        case _ => Nil
      }
    tweetPairsWithWeight.sumByKey
      .flatMap {
        case ((tweetId1, tweetId2), weight) =>
          Seq(
            (tweetId1, TweetWithScore(tweetId2, weight)),
            (tweetId2, TweetWithScore(tweetId1, weight))
          )
        case _ => Nil
      }
  }

  def getPerQueryStatsExec(
    tweetPairs: TypedPipe[(FeaturedTweet, FeaturedTweet, Boolean)],
    outputPath: String,
    identifier: String
  ): Execution[Unit] = {
    val queryTweetsToCounts = tweetPairs
      .map {
        case (queryTweet, _, label) =>
          if (label) (queryTweet.tweet, (1, 0)) else (queryTweet.tweet, (0, 1))
      }
      .sumByKey
      .map { case (queryTweet, (posCount, negCount)) => (queryTweet, posCount, negCount) }

    Execution
      .zip(
        queryTweetsToCounts.writeExecution(
          TypedTsv[(TweetId, Int, Int)](s"${outputPath}_$identifier")),
        Util.printSummaryOfNumericColumn(
          queryTweetsToCounts
            .map { case (_, posCount, _) => posCount },
          Some(s"Per-query Positive Count ($identifier)")),
        Util.printSummaryOfNumericColumn(
          queryTweetsToCounts
            .map { case (_, _, negCount) => negCount },
          Some(s"Per-query Negative Count ($identifier)"))
      ).unit
  }

  def getKeyValTopKSimilarTweets(
    allTweetPairs: TypedPipe[(TweetId, TweetWithScore)],
    k: Int
  )(
  ): TypedPipe[(TweetId, TweetsWithScore)] = {
    allTweetPairs.group
      .sortedReverseTake(k)(Ordering.by(_.score))
      .map { case (tweetId, tweetWithScoreSeq) => (tweetId, TweetsWithScore(tweetWithScoreSeq)) }
  }

  def getTopKSimilarTweets(
    allTweetPairs: TypedPipe[(TweetId, TweetWithScore)],
    k: Int
  )(
  ): TypedPipe[TweetTopKTweetsWithScore] = {
    allTweetPairs.group
      .sortedReverseTake(k)(Ordering.by(_.score))
      .map {
        case (tweetId, tweetWithScoreSeq) =>
          TweetTopKTweetsWithScore(tweetId, TweetsWithScore(tweetWithScoreSeq))
      }
  }

  def allUniquePairs[T](input: Seq[T]): Stream[(T, T)] = {
    input match {
      case Nil => Stream.empty
      case seq =>
        seq.tail.toStream.map(a => (seq.head, a)) #::: allUniquePairs(seq.tail)
    }
  }
}
