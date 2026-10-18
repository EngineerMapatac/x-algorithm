package com.twitter.simclusters_v2.summingbird.common

import com.twitter.algebird.DecayedValue
import com.twitter.algebird.Monoid
import com.twitter.algebird.OptionMonoid
import com.twitter.algebird.ScMapMonoid
import com.twitter.algebird_internal.thriftscala.{DecayedValue => ThriftDecayedValue}
import com.twitter.simclusters_v2.common.SimClustersEmbedding
import com.twitter.simclusters_v2.thriftscala.ClustersWithScores
import com.twitter.simclusters_v2.thriftscala.PersistentSimClustersEmbedding
import com.twitter.simclusters_v2.thriftscala.PersistentTwhinTweetEmbedding
import com.twitter.simclusters_v2.thriftscala.PersistentTwhinUserEmbedding
import com.twitter.simclusters_v2.thriftscala.Scores
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingMetadata
import com.twitter.simclusters_v2.thriftscala.TopKClustersWithScores
import com.twitter.simclusters_v2.thriftscala.TopKHydratedTweetsWithScores
import com.twitter.simclusters_v2.thriftscala.TopKTweetsWithScores
import com.twitter.simclusters_v2.thriftscala.TwhinTweetEmbedding
import com.twitter.simclusters_v2.thriftscala.{SimClustersEmbedding => ThriftSimClustersEmbedding}
import com.twitter.snowflake.id.SnowflakeId

import scala.collection.mutable

object Monoids {

  class ScoresMonoid(implicit thriftDecayedValueMonoid: ThriftDecayedValueMonoid)
      extends Monoid[Scores] {

    private val optionalThriftDecayedValueMonoid =
      new OptionMonoid[ThriftDecayedValue]()

    override val zero: Scores = Scores()

    override def plus(x: Scores, y: Scores): Scores = {
      Scores(
        optionalThriftDecayedValueMonoid.plus(
          x.favClusterNormalized8HrHalfLifeScore,
          y.favClusterNormalized8HrHalfLifeScore
        ),
        optionalThriftDecayedValueMonoid.plus(
          x.followClusterNormalized8HrHalfLifeScore,
          y.followClusterNormalized8HrHalfLifeScore
        )
      )
    }
  }

  class ClustersWithScoresMonoid(implicit scoresMonoid: ScoresMonoid)
      extends Monoid[ClustersWithScores] {

    private val optionMapMonoid =
      new OptionMonoid[collection.Map[Int, Scores]]()(new ScMapMonoid[Int, Scores]())

    override val zero: ClustersWithScores = ClustersWithScores()

    override def plus(x: ClustersWithScores, y: ClustersWithScores): ClustersWithScores = {
      ClustersWithScores(
        optionMapMonoid.plus(x.clustersToScore, y.clustersToScore)
      )
    }
  }

  class TopKClustersWithScoresMonoid(
    topK: Int,
    threshold: Double
  )(
    implicit thriftDecayedValueMonoid: ThriftDecayedValueMonoid)
      extends Monoid[TopKClustersWithScores] {

    override val zero: TopKClustersWithScores = TopKClustersWithScores()

    override def plus(
      x: TopKClustersWithScores,
      y: TopKClustersWithScores
    ): TopKClustersWithScores = {

      val mergedFavMap = TopKScoresUtils
        .mergeTwoTopKMapWithDecayedValues(
          x.topClustersByFavClusterNormalizedScore
            .map(_.mapValues(
              _.favClusterNormalized8HrHalfLifeScore.getOrElse(thriftDecayedValueMonoid.zero))),
          y.topClustersByFavClusterNormalizedScore
            .map(_.mapValues(
              _.favClusterNormalized8HrHalfLifeScore.getOrElse(thriftDecayedValueMonoid.zero))),
          topK,
          threshold
        ).map(_.mapValues(decayedValue =>
          Scores(favClusterNormalized8HrHalfLifeScore = Some(decayedValue))))

      val mergedFollowMap = TopKScoresUtils
        .mergeTwoTopKMapWithDecayedValues(
          x.topClustersByFollowClusterNormalizedScore
            .map(_.mapValues(
              _.followClusterNormalized8HrHalfLifeScore.getOrElse(thriftDecayedValueMonoid.zero))),
          y.topClustersByFollowClusterNormalizedScore
            .map(_.mapValues(
              _.followClusterNormalized8HrHalfLifeScore.getOrElse(thriftDecayedValueMonoid.zero))),
          topK,
          threshold
        ).map(_.mapValues(decayedValue =>
          Scores(followClusterNormalized8HrHalfLifeScore = Some(decayedValue))))

      TopKClustersWithScores(
        mergedFavMap,
        mergedFollowMap
      )
    }
  }
  class TopKTweetsWithScoresMonoid(
    topK: Int,
    threshold: Double,
    tweetAgeThreshold: Long
  )(
    implicit thriftDecayedValueMonoid: ThriftDecayedValueMonoid)
      extends Monoid[TopKTweetsWithScores] {

    override val zero: TopKTweetsWithScores = TopKTweetsWithScores()

    override def plus(x: TopKTweetsWithScores, y: TopKTweetsWithScores): TopKTweetsWithScores = {
      val oldestTweetId = SnowflakeId.firstIdFor(System.currentTimeMillis() - tweetAgeThreshold)

      val mergedFavMap = TopKScoresUtils
        .mergeTwoTopKMapWithDecayedValues(
          x.topTweetsByFavClusterNormalizedScore
            .map(_.mapValues(
              _.favClusterNormalized8HrHalfLifeScore.getOrElse(thriftDecayedValueMonoid.zero))),
          y.topTweetsByFavClusterNormalizedScore
            .map(_.mapValues(
              _.favClusterNormalized8HrHalfLifeScore.getOrElse(thriftDecayedValueMonoid.zero))),
          topK,
          threshold
        ).map(_.filter(_._1 >= oldestTweetId).mapValues(decayedValue =>
          Scores(favClusterNormalized8HrHalfLifeScore = Some(decayedValue))))

      TopKTweetsWithScores(mergedFavMap, None)
    }
  }

  class TopKHydratedTweetsWithScoresMonoid(
    topK: Int,
    threshold: Double,
    tweetAgeThreshold: Long
  )(
    implicit thriftDecayedValueMonoid: ThriftDecayedValueMonoid)
      extends Monoid[TopKHydratedTweetsWithScores] {

    override val zero: TopKHydratedTweetsWithScores = TopKHydratedTweetsWithScores()

    override def plus(
      x: TopKHydratedTweetsWithScores,
      y: TopKHydratedTweetsWithScores
    ): TopKHydratedTweetsWithScores = {
      val oldestTweetId = SnowflakeId.firstIdFor(System.currentTimeMillis() - tweetAgeThreshold)

      val mergedFavMap = TopKScoresUtils
        .mergeTwoTopKMapWithDecayedValues(
          x.topTweetsByFavClusterNormalizedScore
            .map(_.mapValues(
              _.favClusterNormalized8HrHalfLifeScore.getOrElse(thriftDecayedValueMonoid.zero))),
          y.topTweetsByFavClusterNormalizedScore
            .map(_.mapValues(
              _.favClusterNormalized8HrHalfLifeScore.getOrElse(thriftDecayedValueMonoid.zero))),
          topK,
          threshold
        ).map(_.filter(_._1.tweetId >= oldestTweetId).mapValues(decayedValue =>
          Scores(favClusterNormalized8HrHalfLifeScore = Some(decayedValue))))

      TopKHydratedTweetsWithScores(mergedFavMap, None)
    }
  }

  class PersistentSimClustersEmbeddingMonoid extends Monoid[PersistentSimClustersEmbedding] {

    override val zero: PersistentSimClustersEmbedding = PersistentSimClustersEmbedding(
      ThriftSimClustersEmbedding(),
      SimClustersEmbeddingMetadata()
    )

    private val optionLongMonoid = new OptionMonoid[Long]()

    override def plus(
      x: PersistentSimClustersEmbedding,
      y: PersistentSimClustersEmbedding
    ): PersistentSimClustersEmbedding = {
      val latest =
        if (x.metadata.updatedAtMs.getOrElse(0L) > y.metadata.updatedAtMs.getOrElse(0L)) x else y
      latest.copy(
        metadata = latest.metadata.copy(
          updatedCount = optionLongMonoid.plus(x.metadata.updatedCount, y.metadata.updatedCount)))
    }
  }

  class PersistentSimClustersEmbeddingLongestL2NormMonoid
      extends Monoid[PersistentSimClustersEmbedding] {

    override val zero: PersistentSimClustersEmbedding = PersistentSimClustersEmbedding(
      ThriftSimClustersEmbedding(),
      SimClustersEmbeddingMetadata()
    )

    override def plus(
      x: PersistentSimClustersEmbedding,
      y: PersistentSimClustersEmbedding
    ): PersistentSimClustersEmbedding = {
      if (SimClustersEmbedding(x.embedding).l2norm >= SimClustersEmbedding(y.embedding).l2norm) x
      else y
    }
  }

  private class TwhinTweetEmbeddingMonoid(embeddingSize: Int) extends Monoid[TwhinTweetEmbedding] {

    private val emptyEmbedding = Seq.fill(embeddingSize)(0.0)
    override val zero: TwhinTweetEmbedding = TwhinTweetEmbedding(emptyEmbedding)
    override def plus(x: TwhinTweetEmbedding, y: TwhinTweetEmbedding): TwhinTweetEmbedding = {
      val left = if (x.embedding.size == embeddingSize) x else zero
      val right = if (y.embedding.size == embeddingSize) y else zero

      if (left == zero) {
        right
      } else if (right == zero) {
        left
      } else {
        val newEmbedding = (0 until embeddingSize).map { i =>
          left.embedding(i) + right.embedding(i)
        }
        TwhinTweetEmbedding(newEmbedding)
      }
    }

  }

  class PersistentTwhinTweetEmbeddingMonoid(embeddingSize: Int)
      extends Monoid[PersistentTwhinTweetEmbedding] {

    private val tweetEmbeddingMonoid = new TwhinTweetEmbeddingMonoid(embeddingSize)

    override def zero: PersistentTwhinTweetEmbedding = PersistentTwhinTweetEmbedding(
      tweetEmbeddingMonoid.zero,
      updatedCount = 0,
      updatedAt = 0
    )

    override def plus(
      x: PersistentTwhinTweetEmbedding,
      y: PersistentTwhinTweetEmbedding
    ): PersistentTwhinTweetEmbedding = {
      PersistentTwhinTweetEmbedding(
        embedding = tweetEmbeddingMonoid.plus(x.embedding, y.embedding),
        updatedCount = x.updatedCount + y.updatedCount,
        updatedAt = Math.max(x.updatedAt, y.updatedAt),
        authorId = if (x.authorId.isEmpty) y.authorId else x.authorId,
        dataset = if (x.dataset.isEmpty) y.dataset else x.dataset
      )
    }
  }

  class PersistentTwhinUserEmbeddingMonoid(embeddingSize: Int, halfLifeMs: Long, eps: Double)
      extends Monoid[PersistentTwhinUserEmbedding] {

    val EmptyEmbedding: TwhinTweetEmbedding = TwhinTweetEmbedding(Seq.fill(embeddingSize)(0.0))
    override def zero: PersistentTwhinUserEmbedding = PersistentTwhinUserEmbedding(
      EmptyEmbedding,
      updatedAt = 0
    )

    override def plus(
      x: PersistentTwhinUserEmbedding,
      y: PersistentTwhinUserEmbedding
    ): PersistentTwhinUserEmbedding = {
      val oldEmbedding = if (x.updatedAt < y.updatedAt) x else y
      val newEmbedding = if (oldEmbedding == x) y else x

      val oldScaledTime = oldEmbedding.updatedAt * math.log(2.0) / halfLifeMs
      val newScaledTime = newEmbedding.updatedAt * math.log(2.0) / halfLifeMs

      val decay = math.exp(newScaledTime - oldScaledTime)
      val updatedEmbedding =
        (0 until embeddingSize).map { i =>
          val newValue =
            oldEmbedding.embedding.embedding(i) / decay + newEmbedding.embedding.embedding(i)
          if (math.abs(newValue) > eps) newValue else 0.0
        }

      PersistentTwhinUserEmbedding(
        embedding = TwhinTweetEmbedding(updatedEmbedding),
        updatedAt = newEmbedding.updatedAt
      )
    }
  }

  object TopKScoresUtils {

    def mergeClusterScoresWithUpdateTimes[Key](
      x: Seq[(Key, Double)],
      xUpdatedAtMs: Long,
      y: Seq[(Key, Double)],
      yUpdatedAtMs: Long,
      halfLifeMs: Long,
      topK: Int,
      threshold: Double
    ): Seq[(Key, Double)] = {
      val latestUpdate = math.max(xUpdatedAtMs, yUpdatedAtMs)
      val decayedValueForLatestTime = DecayedValue.build(0.0, latestUpdate, halfLifeMs)

      val merged = mutable.HashMap[Key, Double]()

      x.foreach {
        case (key, score) =>
          val decayedScore = Implicits.decayedValueMonoid
            .plus(
              DecayedValue.build(score, xUpdatedAtMs, halfLifeMs),
              decayedValueForLatestTime
            ).value
          if (decayedScore > threshold)
            merged += key -> decayedScore
      }

      y.foreach {
        case (key, score) =>
          val decayedScore = Implicits.decayedValueMonoid
            .plus(
              DecayedValue.build(score, yUpdatedAtMs, halfLifeMs),
              decayedValueForLatestTime
            ).value
          if (decayedScore > threshold)
            merged.get(key) match {
              case Some(existingValue) =>
                if (decayedScore > existingValue)
                  merged += key -> decayedScore
              case None =>
                merged += key -> decayedScore
            }
      }

      merged.toSeq
        .sortBy(-_._2)
        .take(topK)
    }

    def mergeTwoTopKMapWithDecayedValues[T](
      a: Option[collection.Map[T, ThriftDecayedValue]],
      b: Option[collection.Map[T, ThriftDecayedValue]],
      topK: Int,
      threshold: Double
    )(
      implicit thriftDecayedValueMonoid: ThriftDecayedValueMonoid
    ): Option[collection.Map[T, ThriftDecayedValue]] = {

      if (a.isEmpty || a.exists(_.isEmpty)) {
        return b
      }

      if (b.isEmpty || b.exists(_.isEmpty)) {
        return a
      }

      val latestScaledTime = (a.get.view ++ b.get.view).map {
        case (_, scores) =>
          scores.scaledTime
      }.max

      val decayedValueWithLatestScaledTime = ThriftDecayedValue(0.0, latestScaledTime)

      val merged = mutable.HashMap[T, ThriftDecayedValue]()

      a.foreach {
        _.foreach {
          case (k, v) =>
            val decayedScores = thriftDecayedValueMonoid
              .plus(v, decayedValueWithLatestScaledTime)

            if (decayedScores.value > threshold) {
              merged += k -> decayedScores
            }
        }
      }

      b.foreach {
        _.foreach {
          case (k, v) =>
            val decayedScores = thriftDecayedValueMonoid
              .plus(v, decayedValueWithLatestScaledTime)

            if (decayedScores.value > threshold) {
              if (!merged.contains(k)) {
                merged += k -> decayedScores
              } else {
                if (decayedScores.value > merged(k).value) {
                  merged.update(k, decayedScores)
                }
              }
            }
        }
      }

      if (merged.size > topK * 1.2) {
        Some(
          merged.toSeq
            .sortBy { case (_, scores) => scores.value * -1 }
            .take(topK)
            .toMap
        )
      } else {
        Some(merged)
      }
    }
  }
}
