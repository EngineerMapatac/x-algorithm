package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.useng.common.xreview.XReviewReportKeys
import com.twitter.useng.common.xreview.XReviewReportKeys.pb
import intake_service.IntakeService.SubmitReportRequest
import java.lang.{Long => JLong}
import scala.jdk.CollectionConverters._

object XReviewReportBuilder {

  val ReportSurfaceValue = "x_app"
  val ReportSourceValue = "proactive"
  val Subject = "Botmaker report"

  val ReporterIdValue = "0"

  val DescriptionKey = "description"

  val PostEntityType = "post"
  val ProfileEntityType = "profile"
  val SupportedEntityTypes: Set[String] = Set(PostEntityType, ProfileEntityType)

  def normalizeEntityType(entityType: String): Option[String] = {
    Option(entityType).map(_.trim.toLowerCase).filter(SupportedEntityTypes.contains)
  }

  def toSubmitReportRequest(
    entityType: String,
    entityId: Long,
    userId: Long,
    reportType: String,
    botId: Long,
    note: Option[String],
    victimId: Option[JLong],
    detectionTimestampMs: Long
  ): SubmitReportRequest = {
    val reportFields = Seq(
      pb(XReviewReportKeys.Subject, Subject),
      pb(XReviewReportKeys.ReportSurface, ReportSurfaceValue),
      pb(XReviewReportKeys.ReportType, reportType),
      pb(XReviewReportKeys.ReportSource, ReportSourceValue),
      pb(XReviewReportKeys.ReportedEntityType, entityType),
      pb(XReviewReportKeys.ReportedEntityId, entityId.toString),
      pb(XReviewReportKeys.ReportedUserId, userId.toString),
      pb(XReviewReportKeys.DetectionBotId, botId.toString),
      pb(XReviewReportKeys.DetectionTimestampMs, detectionTimestampMs.toString)
    ) ++
      note.filter(_.nonEmpty).map(n => pb(DescriptionKey, n)).toSeq ++
      victimId.filter(_ != null).map(id => pb(XReviewReportKeys.VictimUserId, id.toString)).toSeq

    val reporterFields = Seq(pb(XReviewReportKeys.ReporterId, ReporterIdValue))

    SubmitReportRequest
      .newBuilder()
      .addAllReport(reportFields.asJava)
      .addAllReporter(reporterFields.asJava)
      .build()
  }
}
