package com.twitter.botmaker.app.scarecrow.function

import com.twitter.agenttools.ais.thriftjava.ActorHeader
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit2O1
import com.twitter.botmaker.FunctionUnit2O11
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.app.scarecrow.legacy.DownstreamServices
import com.twitter.botmaker.app.scarecrow.thriftscala.ActionIntakeWithAttributionResponse
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.health.policy.PolicyKey
import com.twitter.util.Future
import java.lang.{Boolean => JBoolean}
import java.lang.{Long => JLong}
import java.util.{List => JList}
import java.util.{Set => JSet}
import scala.collection.JavaConverters._

object TweetRtfApplyLabel
    extends FunctionUnit2O11[ScarecrowRuntime, String, Long, JList[
      JLong
    ], JBoolean, Double, JBoolean, String, Long, String, Long, String, ActorHeader, JSet[
      PolicyKey
    ], Future[
      ActionIntakeWithAttributionResponse
    ]] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.PROD_TANGIBLE_IMPACT_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.ActionIntakeService)
  override def description: String =
    """Applies safety labels to TweetId by calling ActionIntakeService API AddLabelsV2.
      | This function can take *seconds* to return so use with care in synchronous bots. Consider wrapping with DoAsync
      |
      | Parameters:
      | - safetyLabelType: Maps to a value in com.twitter.spam.rtf.thriftjava.SafetyLabelType
      | - tweetId: Id of the tweet being applied label
      | - [Optional] applicableUsers: Applicable users for the label – subset of mentions
      | - [Deprecated] enableHoldback: Enable spam team holdback experiment.
      | - [Optional] score: Score of the Safety label, default None.
      | - [Deprecated] enableVictimSide: enable per-bot victim-side experiment.
      | - [Optional] source: The source of the request.
      | - [Deprecated] modelVersion: Model version (used to popular ModelMetadata)
      | - [Deprecated] calibratedLanguage: Calibrated model language (used to populate ModelMetadata)
      | - [Optional] expiration: expiration timestamp in ms (used for TTL support)
      | - [Deprecated] placeholder for future params, since FunctionUnit2O10 does not exist
      | - [Optional] referrerActor: Identifier for the upstream actor that originally classified the activity as label worthy
      | - [Optional] policyKeys: List of policies that underpins the reason for the suspension action
      |
      |""".stripMargin

  override def arguments: Seq[String] = Seq[String](
    "safetyLabelType",
    "tweetId",
    "applicableUsers",
    "enableHoldback",
    "score",
    "enableVictimSide",
    "source",
    "modelVersion",
    "calibratedLanguage",
    "expiration",
    "placeholder",
    "referrerActor",
    "policyKeys"
  )

  override def evaluate(
    context: Context[ScarecrowRuntime],
    safetyLabelType: String,
    tweetId: Long,
    applicableUsers: Option[JList[JLong]],
    enableHoldback: Option[JBoolean],
    score: Option[Double],
    enableVictimSide: Option[JBoolean],
    source: Option[String],
    modelVersion: Option[Long],
    calibratedLanguage: Option[String],
    expiration: Option[Long],
    placeholder: Option[String],
    referrerActor: Option[ActorHeader],
    policyKeys: Option[JSet[PolicyKey]]
  ): Future[ActionIntakeWithAttributionResponse] = {
    val safetyLabelTypes: Set[String] = Set(safetyLabelType)

    val applicableUsersScalaSet: Option[Set[JLong]] = applicableUsers.map(_.asScala.toSet)
    context.getRuntime.fetcher20.actionIntakeServiceFetcher.applyTweetLabels(
      context = context,
      tweetId = tweetId,
      safetyLabelTypes = safetyLabelTypes,
      policyKeys = policyKeys,
      referrerActor = referrerActor,
      applicableUsers = applicableUsersScalaSet.map(_.map(_.longValue())),
      score = score,
      source = source,
      expirationTimeInMs = expiration
    )
  }
}

object TweetRtfRemoveLabel
    extends FunctionUnit2O1[ScarecrowRuntime, String, Long, String, Future[
      ActionIntakeWithAttributionResponse
    ]] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.PROD_TANGIBLE_IMPACT_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.ActionIntakeService)

  override def description: String =
    """Removes the specified safety label for a given tweet. The returned boolean value should always be set to true.
      |
      | Parameters:
      | - safetyLabelType: Value should map to one in com.twitter.spam.rtf.thriftjava.SafetyLabelType
      | - tweetId: Id of the tweet having label removed
      | - [Optional] source: The source of the request.
      |""".stripMargin

  override def arguments: Seq[String] =
    Seq[String](
      "safetyLabelType",
      "tweetId",
      "source"
    )

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    safetyLabelType: String,
    id: Long,
    source: Option[String]
  ): Future[ActionIntakeWithAttributionResponse] = {
    val safetyLabelTypes: Set[String] = Set(safetyLabelType)

    context.getRuntime.fetcher20.actionIntakeServiceFetcher.removeTweetLabels(
      context = context,
      tweetId = id,
      safetyLabelTypes = safetyLabelTypes,
      source = source
    )
  }
}
