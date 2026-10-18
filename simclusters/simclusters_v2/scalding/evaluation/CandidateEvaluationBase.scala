package com.twitter.simclusters_v2.scalding.evaluation

import com.twitter.core_workflows.user_model.thriftscala.CondensedUserState
import com.twitter.core_workflows.user_model.thriftscala.UserState
import com.twitter.pluck.source.core_workflows.user_model.CondensedUserStateScalaDataset
import com.twitter.scalding._
import com.twitter.scalding.source.TypedText
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.simclusters_v2.thriftscala.CandidateTweets
import com.twitter.simclusters_v2.thriftscala.ReferenceTweets
import scala.util.Random

object UserStateUserSampler {
  def getSampleUsersByUserState(
    userStateSource: TypedPipe[CondensedUserState],
    validStates: Seq[UserState],
    samplePercentage: Double
  ): TypedPipe[(UserState, Long)] = {
    assert(samplePercentage >= 0 && samplePercentage <= 1)
    val validStateSet = validStates.toSet

    userStateSource
      .collect {
        case data if data.userState.isDefined && validStateSet.contains(data.userState.get) =>
          (data.userState.get, data.uid)
      }
      .filter(_ => Random.nextDouble() <= samplePercentage)
      .forceToDisk
  }

  def parseUserStates(strStates: Seq[String]): Seq[UserState] = {
    if (strStates.isEmpty) {
      UserState.list
    } else {
      strStates.map { str =>
        UserState
          .valueOf(str).getOrElse(
            throw new IllegalArgumentException(
              s"Input user_states $str is invalid. Valid states are: " + UserState.list
            )
          )
      }
    }
  }
}

trait UserStateBasedEvaluationExecutionBase
    extends CandidateEvaluationBase
    with TwitterExecutionApp {

  def referenceTweets: TypedPipe[ReferenceTweets]
  def candidateTweets: TypedPipe[CandidateTweets]

  override def job: Execution[Unit] = {
    Execution.withId { implicit uniqueId =>
      Execution.withArgs { args =>
        implicit val dateRange: DateRange =
          DateRange.parse(args.list("date"))(DateOps.UTC, DateParser.default)

        val outputRootDir = args("outputDir")
        val userStates: Seq[UserState] =
          UserStateUserSampler.parseUserStates(args.list("user_states"))
        val sampleRate = args.double("sample_rate")

        val userStateSource = DAL.read(CondensedUserStateScalaDataset).toTypedPipe
        val userIdsByState =
          UserStateUserSampler.getSampleUsersByUserState(userStateSource, userStates, sampleRate)
        val executionsPerUserState = userStates.map { userState =>
          val sampleUsers = userIdsByState.collect { case data if data._1 == userState => data._2 }
          val outputPath = outputRootDir + "/" + userState + "/"

          super
            .runSampledEvaluation(sampleUsers, referenceTweets, candidateTweets)
            .writeExecution(TypedText.csv(outputPath))
        }
        Execution.sequence(executionsPerUserState).unit
      }
    }
  }
}

trait CandidateEvaluationBase {
  private def getSampledReferenceTweets(
    referenceTweetEngagements: TypedPipe[ReferenceTweets],
    sampleUsers: TypedPipe[Long]
  ): TypedPipe[ReferenceTweets] = {
    referenceTweetEngagements
      .groupBy(_.targetUserId)
      .join(sampleUsers.asKeys)
      .map { case (targetUserId, (referenceEngagements, _)) => referenceEngagements }
  }

  private def getSampledCandidateTweets(
    candidateTweets: TypedPipe[CandidateTweets],
    sampleUsers: TypedPipe[Long]
  ): TypedPipe[CandidateTweets] = {
    candidateTweets
      .groupBy(_.targetUserId)
      .join(sampleUsers.asKeys)
      .map { case (_, (tweets, _)) => tweets }
  }

  def evaluateResults(
    sampledReference: TypedPipe[ReferenceTweets],
    sampledCandidate: TypedPipe[CandidateTweets]
  ): TypedPipe[String]

  def runSampledEvaluation(
    targetUserSamples: TypedPipe[Long],
    referenceTweets: TypedPipe[ReferenceTweets],
    candidateTweets: TypedPipe[CandidateTweets]
  ): TypedPipe[String] = {
    val sampledCandidate = getSampledCandidateTweets(candidateTweets, targetUserSamples)
    val referencePerUser = getSampledReferenceTweets(referenceTweets, targetUserSamples)

    evaluateResults(referencePerUser, sampledCandidate)
  }
}
