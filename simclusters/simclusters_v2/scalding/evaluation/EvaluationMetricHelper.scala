package com.twitter.simclusters_v2.scalding.evaluation

import com.twitter.scalding.Execution
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.UniqueID
import com.twitter.simclusters_v2.thriftscala.CandidateTweet
import com.twitter.simclusters_v2.thriftscala.CandidateTweets
import com.twitter.simclusters_v2.thriftscala.ReferenceTweet
import com.twitter.simclusters_v2.thriftscala.ReferenceTweets
import com.twitter.simclusters_v2.thriftscala.TweetLabels
import com.twitter.algebird.Aggregator.size
import com.twitter.scalding.typed.CoGrouped
import com.twitter.scalding.typed.ValuePipe
import com.twitter.util.TwitterDateFormat
import java.util.Calendar

case class UserEngagerCounts(
  numDistinctTargetUsers: Long,
  numDistinctLikeEngagers: Long,
  numDistinctRetweetEngagers: Long)

case class TweetStats(
  numTweets: Long,
  numDistinctTweets: Long,
  numDistinctAuthors: Option[Long],
  avgScore: Option[Double])

case class TweetEngagementCounts(like: Long, retweet: Long, click: Long, hasEngagement: Long)

case class TweetEngagementRates(like: Double, retweet: Double, click: Double, hasEngagement: Double)

case class LabelCorrelations(
  pearsonCoefficientForLikes: Double,
  cosineSimilarityGlobal: Double,
  cosineSimilarityPerUserAvg: Double) {
  private val f = java.text.NumberFormat.getInstance
  def format(): String = {
    Seq(
      s"\tPearson Coefficient: ${f.format(pearsonCoefficientForLikes)}",
      s"\tCosine similarity: ${f.format(cosineSimilarityGlobal)}",
      s"\tAverage cosine similarity for all users: ${f.format(cosineSimilarityPerUserAvg)}"
    ).mkString("\n")
  }
}

case class LabeledTweet(
  targetUserId: Long,
  tweetId: Long,
  authorId: Long,
  labels: TweetLabels,
  algorithmScore: Option[Double])

case class LabeledTweetsResults(
  tweetStats: TweetStats,
  userEngagerCounts: UserEngagerCounts,
  tweetEngagementCounts: TweetEngagementCounts,
  tweetEngagementRates: TweetEngagementRates,
  labelCorrelations: Option[LabelCorrelations] = None) {
  private val f = java.text.NumberFormat.getInstance

  def format(title: String = ""): String = {
    val str = Seq(
      s"Number of tweets: ${f.format(tweetStats.numTweets)}",
      s"Number of distinct tweets: ${f.format(tweetStats.numDistinctTweets)}",
      s"Number of distinct users targeted: ${f.format(userEngagerCounts.numDistinctTargetUsers)}",
      s"Number of distinct authors: ${tweetStats.numDistinctAuthors.map(f.format).getOrElse("N/A")}",
      s"Average algorithm score of tweets: ${tweetStats.avgScore.map(f.format).getOrElse("N/A")}",
      s"Engager counts:",
      s"\tNumber of users who liked tweets: ${f.format(userEngagerCounts.numDistinctLikeEngagers)}",
      s"\tNumber of users who retweeted tweets: ${f.format(userEngagerCounts.numDistinctRetweetEngagers)}",
      s"Tweet engagement counts:",
      s"\tNumber of Likes: ${f.format(tweetEngagementCounts.like)}",
      s"\tNumber of Retweets: ${f.format(tweetEngagementCounts.retweet)}",
      s"\tNumber of Clicks: ${f.format(tweetEngagementCounts.click)}",
      s"\tNumber of tweets with any engagements: ${f.format(tweetEngagementCounts.hasEngagement)}",
      s"Tweet engagement rates:",
      s"\tRate of Likes: ${f.format(tweetEngagementRates.like * 100)}%",
      s"\tRate of Retweets: ${f.format(tweetEngagementRates.retweet * 100)}%",
      s"\tRate of Clicks: ${f.format(tweetEngagementRates.click * 100)}%",
      s"\tRate of any engagement: ${f.format(tweetEngagementRates.hasEngagement * 100)}%"
    ).mkString("\n")

    val correlations = labelCorrelations.map("\n" + _.format()).getOrElse("")

    s"$title\n$str$correlations"
  }
}

case class CandidateResults(tweetStats: TweetStats, numDistinctTargetUsers: Long) {
  private val f = java.text.NumberFormat.getInstance

  def format(title: String = ""): String = {
    val str = Seq(
      s"Number of tweets: ${f.format(tweetStats.numTweets)}",
      s"Number of distinct tweets: ${f.format(tweetStats.numDistinctTweets)}",
      s"Number of distinct users targeted: ${f.format(numDistinctTargetUsers)}",
      s"Number of distinct authors: ${tweetStats.numDistinctAuthors.map(f.format).getOrElse("N/A")}",
      s"Average algorithm score of tweets: ${tweetStats.avgScore.map(f.format).getOrElse("N/A")}"
    ).mkString("\n")
    s"$title\n$str"
  }
}

object EvaluationMetricHelper {
  private def toLong(bool: Boolean): Long = {
    if (bool) 1L else 0L
  }

  private def hasCoreEngagements(labels: TweetLabels): Boolean = {
    labels.isRetweeted ||
    labels.isLiked ||
    labels.isQuoted ||
    labels.isReplied
  }

  private def hasCoreEngagementsOrClick(labels: TweetLabels): Boolean = {
    hasCoreEngagements(labels) || labels.isClicked
  }

  def outerJoinReferenceAndCandidate(
    referencePipe: TypedPipe[ReferenceTweets],
    candidatePipe: TypedPipe[CandidateTweets]
  ): CoGrouped[(Long, Long), (Option[ReferenceTweet], Option[CandidateTweet])] = {

    val references = referencePipe
      .flatMap { refTweets =>
        refTweets.impressedTweets.map { refTweet =>
          ((refTweets.targetUserId, refTweet.tweetId), refTweet)
        }
      }

    val candidates = candidatePipe
      .flatMap { candTweets =>
        candTweets.recommendedTweets.map { candTweet =>
          ((candTweets.targetUserId, candTweet.tweetId), candTweet)
        }
      }

    references.outerJoin(candidates).withReducers(50)
  }

  def getLabeledReference(referencePipe: TypedPipe[ReferenceTweets]): TypedPipe[LabeledTweet] = {
    referencePipe
      .flatMap { refTweets =>
        refTweets.impressedTweets.map { tweet =>
          LabeledTweet(refTweets.targetUserId, tweet.tweetId, tweet.authorId, tweet.labels, None)
        }
      }
  }

  def getUniqueCount[T](pipe: TypedPipe[T])(implicit ord: scala.Ordering[T]): Execution[Long] = {
    pipe.distinct
      .aggregate(size)
      .toOptionExecution
      .map(_.getOrElse(0L))
  }

  def countUniqueEngagedUsersBy(
    labeledTweetsPipe: TypedPipe[LabeledTweet],
    f: TweetLabels => Boolean
  ): Execution[Long] = {
    getUniqueCount[Long](labeledTweetsPipe.collect { case t if f(t.labels) => t.targetUserId })
  }

  def countUniqueLabeledTargetUsers(labeledTweetsPipe: TypedPipe[LabeledTweet]): Execution[Long] = {
    getUniqueCount[Long](labeledTweetsPipe.map(_.targetUserId))
  }

  def countUniqueCandTargetUsers(candidatePipe: TypedPipe[CandidateTweets]): Execution[Long] = {
    getUniqueCount[Long](candidatePipe.map(_.targetUserId))
  }

  def countUniqueLabeledAuthors(labeledTweetPipe: TypedPipe[LabeledTweet]): Execution[Long] = {
    getUniqueCount[Long](labeledTweetPipe.map(_.authorId))
  }

  def getEngagementRate(
    basicStats: TweetStats,
    engagementCount: TweetEngagementCounts
  ): TweetEngagementRates = {
    val numTweets = basicStats.numTweets.toDouble
    if (numTweets <= 0) throw new IllegalArgumentException("Invalid tweet counts")
    val likeRate = engagementCount.like / numTweets
    val rtRate = engagementCount.retweet / numTweets
    val clickRate = engagementCount.click / numTweets
    val engagementRate = engagementCount.hasEngagement / numTweets
    TweetEngagementRates(likeRate, rtRate, clickRate, engagementRate)
  }

  def getTweetStatsForCandidateExec(
    candidatePipe: TypedPipe[CandidateTweets]
  ): Execution[TweetStats] = {
    val pipe = candidatePipe.map { candTweets =>
      (candTweets.targetUserId, candTweets.recommendedTweets)
    }.sumByKey

    val distinctTweetPipe = pipe.flatMap(_._2.map(_.tweetId)).distinct.aggregate(size)

    val otherStats = pipe
      .map {
        case (uid, recommendedTweets) =>
          val scoreSum = recommendedTweets.flatMap(_.score).sum
          (recommendedTweets.size.toLong, scoreSum)
      }
      .sum
      .map {
        case (numTweets, scoreSum) =>
          if (numTweets <= 0) throw new IllegalArgumentException("Invalid tweet counts")
          val avgScore = scoreSum / numTweets.toDouble
          (numTweets, avgScore)
      }
    ValuePipe
      .fold(distinctTweetPipe, otherStats) {
        case (numDistinctTweet, (numTweets, avgScore)) =>
          TweetStats(numTweets, numDistinctTweet, None, Some(avgScore))
      }.getOrElseExecution(TweetStats(0L, 0L, None, None))
  }

  def getLabeledEngagementCountExec(
    labeledTweets: TypedPipe[LabeledTweet]
  ): Execution[TweetEngagementCounts] = {
    labeledTweets
      .map { labeledTweet =>
        val like = toLong(labeledTweet.labels.isLiked)
        val retweet = toLong(labeledTweet.labels.isRetweeted)
        val click = toLong(labeledTweet.labels.isClicked)
        val hasEngagement = toLong(hasCoreEngagementsOrClick(labeledTweet.labels))

        (like, retweet, click, hasEngagement)
      }
      .sum
      .map {
        case (like, retweet, click, hasEngagement) =>
          TweetEngagementCounts(like, retweet, click, hasEngagement)
      }
      .getOrElseExecution(TweetEngagementCounts(0L, 0L, 0L, 0L))
  }

  def getTargetUserStatsForLabeledTweetsExec(
    labeledTweetsPipe: TypedPipe[LabeledTweet]
  ): Execution[UserEngagerCounts] = {
    val numUniqueTargetUsersExec = countUniqueLabeledTargetUsers(labeledTweetsPipe)
    val numUniqueLikeUsersExec =
      countUniqueEngagedUsersBy(labeledTweetsPipe, labels => labels.isLiked)
    val numUniqueRetweetUsersExec =
      countUniqueEngagedUsersBy(labeledTweetsPipe, labels => labels.isRetweeted)

    Execution
      .zip(
        numUniqueTargetUsersExec,
        numUniqueLikeUsersExec,
        numUniqueRetweetUsersExec
      )
      .map {
        case (numTarget, like, retweet) =>
          UserEngagerCounts(
            numDistinctTargetUsers = numTarget,
            numDistinctLikeEngagers = like,
            numDistinctRetweetEngagers = retweet
          )
      }
  }

  def getTweetStatsForLabeledTweetsExec(
    labeledTweetPipe: TypedPipe[LabeledTweet]
  ): Execution[TweetStats] = {
    val uniqueAuthorsExec = countUniqueLabeledAuthors(labeledTweetPipe)

    val uniqueTweetExec =
      labeledTweetPipe.map(_.tweetId).distinct.aggregate(size).getOrElseExecution(0L)
    val scoresExec = labeledTweetPipe
      .map { t => (t.targetUserId, (1, t.algorithmScore.getOrElse(0.0))) }
      .sumByKey
      .map {
        case (uid, (c1, c2)) =>
          (c1.toLong, c2)
      }
      .sum
      .map {
        case (numTweets, scoreSum) =>
          if (numTweets <= 0) throw new IllegalArgumentException("Invalid tweet counts")
          val avgScore = scoreSum / numTweets.toDouble
          (numTweets, Option(avgScore))
      }
      .getOrElseExecution((0L, None))

    Execution
      .zip(uniqueAuthorsExec, uniqueTweetExec, scoresExec)
      .map {
        case (numDistinctAuthors, numUniqueTweets, (numTweets, avgScores)) =>
          TweetStats(numTweets, numUniqueTweets, Some(numDistinctAuthors), avgScores)
      }
  }

  private def printOnCompleteMsg(stepDescription: String, startTimeMillis: Long): Unit = {
    val formatDate = TwitterDateFormat("yyyy-MM-dd hh:mm:ss")
    val now = Calendar.getInstance().getTime

    val secondsSpent = (now.getTime - startTimeMillis) / 1000
    println(
      s"- ${formatDate.format(now)}\tStep complete: $stepDescription\t " +
        s"Time spent: ${secondsSpent / 60}m${secondsSpent % 60}s"
    )
  }

  private def getEvaluationResultsForCandidates(
    candidatePipe: TypedPipe[CandidateTweets]
  ): Execution[CandidateResults] = {
    val tweetStatsExec = getTweetStatsForCandidateExec(candidatePipe)
    val numDistinctTargetUsersExec = countUniqueCandTargetUsers(candidatePipe)

    Execution
      .zip(tweetStatsExec, numDistinctTargetUsersExec)
      .map {
        case (tweetStats, numDistinctTargetUsers) =>
          CandidateResults(tweetStats, numDistinctTargetUsers)
      }
  }

  private def getEvaluationResultsForLabeledTweets(
    labeledTweetPipe: TypedPipe[LabeledTweet],
    getLabelCorrelations: Boolean = false
  ): Execution[LabeledTweetsResults] = {
    val tweetStatsExec = getTweetStatsForLabeledTweetsExec(labeledTweetPipe)
    val userStatsExec = getTargetUserStatsForLabeledTweetsExec(labeledTweetPipe)
    val engagementCountExec = getLabeledEngagementCountExec(labeledTweetPipe)

    val correlationsExec = if (getLabelCorrelations) {
      Execution
        .zip(
          LabelCorrelationsHelper.pearsonCoefficientForLike(labeledTweetPipe),
          LabelCorrelationsHelper.cosineSimilarityForLike(labeledTweetPipe),
          LabelCorrelationsHelper.cosineSimilarityForLikePerUser(labeledTweetPipe)
        ).map {
          case (pearsonCoeff, globalCos, avgCos) =>
            Some(LabelCorrelations(pearsonCoeff, globalCos, avgCos))
        }
    } else {
      ValuePipe(None).getOrElseExecution(None)
    }

    Execution
      .zip(tweetStatsExec, engagementCountExec, userStatsExec, correlationsExec)
      .map {
        case (tweetStats, engagementCount, engagerCount, correlationsOpt) =>
          val engagementRate = getEngagementRate(tweetStats, engagementCount)
          LabeledTweetsResults(
            tweetStats,
            engagerCount,
            engagementCount,
            engagementRate,
            correlationsOpt)
      }
  }

  private def runAllEvalForCandidates(
    candidatePipe: TypedPipe[CandidateTweets],
    outerJoinPipe: TypedPipe[((Long, Long), (Option[ReferenceTweet], Option[CandidateTweet]))]
  ): Execution[(CandidateResults, CandidateResults)] = {
    val t0 = System.currentTimeMillis()

    val candidateNotInIntersectionPipe =
      outerJoinPipe
        .collect {
          case ((targetUserId, _), (None, Some(candTweet))) => (targetUserId, Seq(candTweet))
        }
        .sumByKey
        .map { case (targetUserId, candTweets) => CandidateTweets(targetUserId, candTweets) }
        .forceToDisk

    Execution
      .zip(
        getEvaluationResultsForCandidates(candidatePipe),
        getEvaluationResultsForCandidates(candidateNotInIntersectionPipe)
      ).onComplete(_ => printOnCompleteMsg("runAllEvalForCandidates()", t0))
  }

  private def runAllEvalForIntersection(
    outerJoinPipe: TypedPipe[((Long, Long), (Option[ReferenceTweet], Option[CandidateTweet]))]
  )(
    implicit uniqueID: UniqueID
  ): Execution[(LabeledTweetsResults, LabeledTweetsResults, LabeledTweetsResults)] = {
    val t0 = System.currentTimeMillis()
    val intersectionTweetsPipe = outerJoinPipe.collect {
      case ((targetUserId, tweetId), (Some(refTweet), Some(candTweet))) =>
        LabeledTweet(targetUserId, tweetId, refTweet.authorId, refTweet.labels, candTweet.score)
    }.forceToDisk

    val likedTweetsPipe = intersectionTweetsPipe.filter(_.labels.isLiked)
    val notLikedTweetsPipe = intersectionTweetsPipe.filter(!_.labels.isLiked)

    Execution
      .zip(
        getEvaluationResultsForLabeledTweets(intersectionTweetsPipe, getLabelCorrelations = true),
        getEvaluationResultsForLabeledTweets(likedTweetsPipe),
        getEvaluationResultsForLabeledTweets(notLikedTweetsPipe)
      ).onComplete(_ => printOnCompleteMsg("runAllEvalForIntersection()", t0))
  }

  private def runAllEvalForReferences(
    referencePipe: TypedPipe[ReferenceTweets],
    outerJoinPipe: TypedPipe[((Long, Long), (Option[ReferenceTweet], Option[CandidateTweet]))]
  ): Execution[(LabeledTweetsResults, LabeledTweetsResults)] = {
    val t0 = System.currentTimeMillis()
    val labeledReferenceNotInIntersectionPipe =
      outerJoinPipe.collect {
        case ((targetUserId, _), (Some(refTweet), None)) =>
          LabeledTweet(targetUserId, refTweet.tweetId, refTweet.authorId, refTweet.labels, None)
      }.forceToDisk

    Execution
      .zip(
        getEvaluationResultsForLabeledTweets(getLabeledReference(referencePipe)),
        getEvaluationResultsForLabeledTweets(labeledReferenceNotInIntersectionPipe)
      ).onComplete(_ => printOnCompleteMsg("runAllEvalForReferences()", t0))
  }

  def runAllEvaluations(
    referencePipe: TypedPipe[ReferenceTweets],
    candidatePipe: TypedPipe[CandidateTweets]
  )(
    implicit uniqueID: UniqueID
  ): Execution[String] = {
    val t0 = System.currentTimeMillis()

    Execution
      .zip(
        referencePipe.forceToDiskExecution,
        candidatePipe.forceToDiskExecution
      ).flatMap {
        case (referenceDiskPipe, candidateDiskPipe) =>
          outerJoinReferenceAndCandidate(referenceDiskPipe, candidateDiskPipe).forceToDiskExecution
            .flatMap { outerJoinPipe =>
              val referenceResultsExec = runAllEvalForReferences(referenceDiskPipe, outerJoinPipe)
              val intersectionResultsExec = runAllEvalForIntersection(outerJoinPipe)
              val candidateResultsExec = runAllEvalForCandidates(candidateDiskPipe, outerJoinPipe)

              Execution
                .zip(
                  referenceResultsExec,
                  intersectionResultsExec,
                  candidateResultsExec
                ).map {
                  case (
                        (allReference, referenceNotInIntersection),
                        (allIntersection, intersectionLiked, intersectionNotLiked),
                        (allCandidate, candidateNotInIntersection)) =>
                    val timeSpent = (System.currentTimeMillis() - t0) / 1000
                    val resultStr = Seq(
                      "===================================================",
                      s"Evaluation complete. Took ${timeSpent / 60}m${timeSpent % 60}s ",
                      allReference.format("-----Metrics for all Reference Tweets-----"),
                      referenceNotInIntersection.format(
                        "-----Metrics for Reference Tweets that are not in the intersection-----"
                      ),
                      allIntersection.format("-----Metrics for all Intersection Tweets-----"),
                      intersectionLiked.format("-----Metrics for Liked Intersection Tweets-----"),
                      intersectionNotLiked.format(
                        "-----Metrics for not Liked Intersection Tweets-----"),
                      allCandidate.format("-----Metrics for all Candidate Tweets-----"),
                      candidateNotInIntersection.format(
                        "-----Metrics for Candidate Tweets that are not in the intersection-----"
                      ),
                      "===================================================\n"
                    ).mkString("\n")
                    println(resultStr)
                    resultStr
                }
                .onComplete(_ =>
                  printOnCompleteMsg(
                    "Evaluation complete. Check stdout or output logs for results.",
                    t0))
            }
      }
  }
}
