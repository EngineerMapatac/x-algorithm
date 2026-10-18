package com.twitter.visibility.under_the_hood

import com.twitter.guano.thriftscala.DmcaTakedown
import com.twitter.guano.thriftscala.DmcaTakedownType
import com.twitter.guano.thriftscala.PctdAction
import com.twitter.guano.thriftscala.PctdActionType
import com.twitter.tweetypie.thriftscala.TweetEvent
import com.twitter.tweetypie.thriftscala.TweetEventData
import com.twitter.visibility.under_the_hood.TakedownLabels.Reason

sealed trait TakedownObservation {
  def userId: Long
  def tweetId: Long
  def eventMs: Long
  def toTsvRow: TakedownObservation.TsvRow
}

case class TakedownDelta(
  userId: Long,
  tweetId: Long,
  eventMs: Long,
  reason: Reason,
  withheld: Boolean,
  mediaId: Option[Long])
    extends TakedownObservation {
  def toTsvRow: TakedownObservation.TsvRow =
    (
      userId,
      tweetId,
      eventMs,
      0,
      TakedownLabels.encodeReason(reason),
      withheld,
      mediaId.getOrElse(0L))
}

case class TakedownState(userId: Long, tweetId: Long, eventMs: Long, reasons: Set[Reason])
    extends TakedownObservation {
  def toTsvRow: TakedownObservation.TsvRow =
    (
      userId,
      tweetId,
      eventMs,
      1,
      TakedownLabels.encodeReasons(reasons.toSeq.sortBy(_.sortKey)),
      reasons.nonEmpty,
      0L)
}

object TakedownObservation {
  type TsvRow = (Long, Long, Long, Int, String, Boolean, Long)

  def fromTsvRow(row: TsvRow): Option[TakedownObservation] = {
    val (userId, tweetId, eventMs, kind, payload, withheld, mediaId) = row
    kind match {
      case 0 =>
        TakedownLabels.decodeReason(payload).map { reason =>
          TakedownDelta(
            userId,
            tweetId,
            eventMs,
            reason,
            withheld,
            if (mediaId != 0L) Some(mediaId) else None)
        }
      case 1 =>
        Some(TakedownState(userId, tweetId, eventMs, TakedownLabels.decodeReasons(payload).toSet))
      case _ => None
    }
  }
}

object TakedownDeltas {
  import UnderTheHoodCommon._

  private[under_the_hood] def fromPctdAction(
    action: PctdAction,
    testUserIds: Set[Long]
  ): Option[TakedownDelta] =
    if (action.`type` != PctdActionType.Status || !inScope(testUserIds, action.userId)) None
    else
      for {
        tweetId <- action.tweetId
        reason <- action.reason match {
          case Some(arm) => TakedownLabels.fromWithholdingArm(arm)
          case None if action.countryCode.trim.nonEmpty =>
            Some(TakedownLabels.unspecified(action.countryCode))
          case None => None
        }
      } yield TakedownDelta(
        userId = action.userId,
        tweetId = tweetId,
        eventMs = action.timestamp.toLong * 1000L,
        reason = reason,
        withheld = action.takendown,
        mediaId = None
      )

  private val UisServiceIdentifier = "user-image-service"

  private val ContentPattern = """(?is)^\((dmca|undo_dmca)\) \[(\d+):[^\]]*\].*""".r

  private[under_the_hood] def parseMediaContent(content: String): Option[(Long, Boolean)] =
    content match {
      case ContentPattern(action, mediaId) =>
        scala.util.Try(mediaId.toLong).toOption.map(id => (id, action.toLowerCase == "dmca"))
      case _ => None
    }

  private[under_the_hood] def isCopyrightViolationRow(audit: DmcaTakedown): Boolean =
    audit.fingerprintId.isDefined ||
      audit.copyrightViolationInfo.isDefined ||
      audit.byAutomatedServiceIdentifier.contains(UisServiceIdentifier)

  private[under_the_hood] def fromDmcaAudit(
    audit: DmcaTakedown,
    testUserIds: Set[Long]
  ): Option[TakedownDelta] =
    if (audit.`type` != DmcaTakedownType.Media ||
      isCopyrightViolationRow(audit) ||
      !inScope(testUserIds, audit.userId)) None
    else
      for {
        tweetId <- audit.tweetId
        content <- audit.content
        (mediaId, dmca) <- parseMediaContent(content)
      } yield TakedownDelta(
        userId = audit.userId,
        tweetId = tweetId,
        eventMs = audit.timestamp.toLong * 1000L,
        reason = TakedownLabels.MediaDmca,
        withheld = dmca,
        mediaId = Some(mediaId)
      )
}

object TakedownStates {
  import UnderTheHoodCommon._

  private[under_the_hood] def fromTweetEvent(
    event: TweetEvent,
    testUserIds: Set[Long]
  ): Option[TakedownState] =
    event.data match {
      case TweetEventData.TweetTakedownEvent(td) if inScope(testUserIds, td.userId) =>
        Some(
          TakedownState(
            userId = td.userId,
            tweetId = td.tweetId,
            eventMs = event.flags.timestampMs,
            reasons = td.takedownReasons.flatMap(TakedownLabels.fromWithholdingArm).toSet
          ))
      case _ => None
    }

  type TsvRow = (Long, Long, Long, String)

  def toTsvRow(st: TakedownState): TsvRow =
    (
      st.userId,
      st.tweetId,
      st.eventMs,
      TakedownLabels.encodeReasons(st.reasons.toSeq.sortBy(_.sortKey)))

  def fromTsvRow(row: TsvRow): TakedownState = {
    val (userId, tweetId, eventMs, encoded) = row
    TakedownState(userId, tweetId, eventMs, TakedownLabels.decodeReasons(encoded).toSet)
  }
}
