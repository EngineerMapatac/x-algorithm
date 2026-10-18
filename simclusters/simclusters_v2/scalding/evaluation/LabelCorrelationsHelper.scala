package com.twitter.simclusters_v2.scalding.evaluation

import com.twitter.algebird.AveragedValue
import com.twitter.scalding.Execution
import com.twitter.scalding.typed.TypedPipe
import com.twitter.simclusters_v2.scalding.common.Util

object LabelCorrelationsHelper {

  private def toDouble(bool: Boolean): Double = {
    if (bool) 1.0 else 0.0
  }

  def cosineSimilarityForLike(labeledTweets: TypedPipe[LabeledTweet]): Execution[Double] = {
    labeledTweets
      .map { tweet => (toDouble(tweet.labels.isLiked), tweet.algorithmScore.getOrElse(0.0)) }
      .toIterableExecution.map { iter => Util.cosineSimilarity(iter.iterator) }
  }

  def cosineSimilarityForLikePerUser(labeledTweets: TypedPipe[LabeledTweet]): Execution[Double] = {
    val avg = AveragedValue.aggregator.composePrepare[(Unit, Double)](_._2)

    labeledTweets
      .map { tweet =>
        (
          tweet.targetUserId,
          Seq((toDouble(tweet.labels.isLiked), tweet.algorithmScore.getOrElse(0.0)))
        )
      }
      .sumByKey
      .map {
        case (userId, seq) =>
          ((), Util.cosineSimilarity(seq.iterator))
      }
      .aggregate(avg)
      .getOrElseExecution(0.0)
  }

  def pearsonCoefficientForLike(labeledTweets: TypedPipe[LabeledTweet]): Execution[Double] = {
    labeledTweets
      .map { tweet => (toDouble(tweet.labels.isLiked), tweet.algorithmScore.getOrElse(0.0)) }
      .toIterableExecution.map { iter => Util.computeCorrelation(iter.iterator) }
  }
}
