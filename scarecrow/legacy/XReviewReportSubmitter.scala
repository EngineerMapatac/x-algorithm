package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.finagle.stats.StatsReceiver
import com.twitter.useng.common.xreview.XReviewIntakeClient
import com.twitter.util.Future
import com.twitter.util.logging.Logging
import java.lang.{Long => JLong}
import scala.util.control.NonFatal

class XReviewReportSubmitter(
  prodClient: Option[XReviewIntakeClient],
  stagingClient: Option[XReviewIntakeClient],
  statsReceiver: StatsReceiver)
    extends Logging {

  private val scoped = statsReceiver.scope("xreview", "botmaker")
  private val attemptedCounter = scoped.counter("attempted")
  private val submittedCounter = scoped.counter("submitted")
  private val skippedCounter = scoped.counter("skipped")
  private val unsupportedEntityCounter = scoped.counter("unsupported_entity")
  private val rpcFailedCounter = scoped.counter("rpc_failed")

  def submit(
    entityType: String,
    entityId: Long,
    userId: Long,
    reportType: String,
    botId: Long,
    note: Option[String],
    victimId: Option[JLong],
    staging: Boolean,
    detectionTimestampMs: Long
  ): Future[Unit] = {
    attemptedCounter.incr()
    XReviewReportBuilder.normalizeEntityType(entityType) match {
      case None =>
        unsupportedEntityCounter.incr()
        Future.exception(
          new IllegalArgumentException(
            s"XReview entity type must be post or profile, got '$entityType'"))
      case Some(normalizedType) =>
        clientFor(staging) match {
          case None =>
            skippedCounter.incr()
            Future.Unit
          case Some(client) =>
            val request = XReviewReportBuilder.toSubmitReportRequest(
              entityType = normalizedType,
              entityId = entityId,
              userId = userId,
              reportType = reportType,
              botId = botId,
              note = note,
              victimId = victimId,
              detectionTimestampMs = detectionTimestampMs
            )
            client
              .submitReport(request)
              .onSuccess(_ => submittedCounter.incr())
              .unit
              .rescue {
                case NonFatal(e) =>
                  rpcFailedCounter.incr()
                  warn(s"[xreview] botmaker report submit failed: ${e.getMessage}", e)
                  Future.exception(e)
              }
        }
    }
  }

  def close(): Unit = {
    prodClient.foreach(_.shutdown())
    stagingClient.foreach(_.shutdown())
  }

  private def clientFor(staging: Boolean): Option[XReviewIntakeClient] =
    if (staging) stagingClient else prodClient
}

object XReviewReportSubmitter {

  def apply(
    prodClient: XReviewIntakeClient,
    stagingClient: XReviewIntakeClient,
    statsReceiver: StatsReceiver
  ): XReviewReportSubmitter =
    new XReviewReportSubmitter(Some(prodClient), Some(stagingClient), statsReceiver)

  def noop(statsReceiver: StatsReceiver): XReviewReportSubmitter =
    new XReviewReportSubmitter(None, None, statsReceiver)
}
