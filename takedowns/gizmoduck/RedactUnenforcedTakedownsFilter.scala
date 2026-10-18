package com.twitter.gizmoduck.filter

import com.twitter.finagle.stats.StatsReceiver
import com.twitter.gizmoduck.request.GetRequest
import com.twitter.gizmoduck.request.Request
import com.twitter.gizmoduck.request.RequestFilter
import com.twitter.gizmoduck.request.RequestHandler
import com.twitter.gizmoduck.thriftscala.Takedown
import com.twitter.gizmoduck.thriftscala.User
import com.twitter.gizmoduck.thriftscala.UserResult
import com.twitter.servo.util.Gate
import com.twitter.tseng.withholding.thriftscala.BystanderReport
import com.twitter.tseng.withholding.thriftscala.LegalRequest
import com.twitter.tseng.withholding.thriftscala.TakedownReason
import com.twitter.tseng.withholding.thriftscala.UnspecifiedReason
import com.twitter.util.Future

class RedactUnenforcedTakedownsFilter(
  enableReadGate: Gate[Unit],
  statsReceiver: StatsReceiver)
    extends RequestFilter {
  import RedactUnenforcedTakedownsFilter._

  private[this] val redacted = statsReceiver.counter("redacted_unenforced_takedowns")

  private[filter] def redactUnenforcedTakedowns(user: User): User =
    user.takedowns match {
      case Some(takedowns) =>
        val (redactedTakedowns, keptTakedowns) =
          takedowns.takedownCountryReasons.getOrElse(Nil).partition(isUnenforcedTakedown)
        if (redactedTakedowns.isEmpty) {
          user
        } else {
          redacted.incr(redactedTakedowns.size)
          user.copy(
            takedowns = Some(takedowns.copy(takedownCountryReasons = Some(keptTakedowns)))
          )
        }
      case None => user
    }

  private[this] def filterGetResults(results: Seq[UserResult]): Seq[UserResult] =
    if (!enableReadGate()) results
    else
      results.map { result =>
        result.user match {
          case Some(user) => result.copy(user = Some(redactUnenforcedTakedowns(user)))
          case None => result
        }
      }

  override def apply[Response](
    request: Request[Response],
    handler: RequestHandler
  ): Future[Response] =
    handler(request).map { response =>
      request match {
        case _: GetRequest[_] => filterGetResults(response)
        case _ => response
      }
    }
}

object RedactUnenforcedTakedownsFilter {
  val WorldwideCountryCode = "xx"
  val WorldwideCopyrightCountryCode = "xy"

  def isUnenforcedTakedown(takedown: Takedown): Boolean =
    takedown.takedownReason match {
      case TakedownReason.UnspecifiedReason(UnspecifiedReason(cc)) =>
        isCode(cc, WorldwideCopyrightCountryCode)
      case TakedownReason.LegalRequest(LegalRequest(cc)) =>
        isCode(cc, WorldwideCopyrightCountryCode)
      case TakedownReason.BystanderReport(BystanderReport(cc)) =>
        isCode(cc, WorldwideCountryCode) || isCode(cc, WorldwideCopyrightCountryCode)
      case TakedownReason.Dmca(_) => true
      case _ => false
    }

  private def isCode(cc: String, code: String): Boolean = cc.trim.equalsIgnoreCase(code)
}
