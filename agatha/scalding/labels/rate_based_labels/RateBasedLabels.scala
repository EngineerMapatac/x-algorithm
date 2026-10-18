package com.twitter.agatha.scalding.labels.rate_based_labels

import com.twitter.abuse.detection.mention_interactions.MentionInteractionsScalaDataset
import com.twitter.abuse.detection.mention_interactions.thriftscala.EngagementType
import com.twitter.agatha.scalding.labels.LabelUtil._
import com.twitter.agatha.scalding.labels.LabelledUsers
import com.twitter.agatha.scalding.labels.blocks_per_fav._
import com.twitter.agatha.scalding.labels.favs.Favs._
import com.twitter.agatha.scalding.labels.reports_per_fav._
import com.twitter.agatha.scalding.labels.spam_reports_per_fav._
import com.twitter.common_internal.analytics.artificial_user_filter.ArtificialUserFilter
import com.twitter.hub.agatha.thriftscala.UserLabel
import com.twitter.scalding.serialization.RequiredBinaryComparators._
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.Execution
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.spam.scalding.datasets.TattleEventsScalaDataset
import com.twitter.spam.thriftscala.TattleType
import com.twitter.useng.rtp.core.thriftscala.Tool
import graphstore.common.FlockBlocksJavaDataset
import java.util.TimeZone
import twadoop_config.configuration.log_categories.group.useng.RtpReportsScalaDataset

object RateBasedLabels {
  def computeScoresForInteractions(
    interactionSizes: TypedPipe[(Long, Long)],
    favSizes: TypedPipe[(Long, Long)],
    smoothingParam: Double,
    labelName: String
  ): TypedPipe[UserLabel] = {
    val interactionAndFavs = favSizes
      .outerJoin(interactionSizes)
      .mapValues {
        case (favOpt, interactionOpt) => (interactionOpt.getOrElse(0L), favOpt.getOrElse(0L))
      }
      .forceToDisk

    val interactionPerFavPrior = getPrior(capFeatureSum(interactionAndFavs.values))

    interactionAndFavs
      .cross(interactionPerFavPrior)
      .map {
        case ((userId, (interaction, favs)), prior) =>
          val numerator = interaction + smoothingParam
          val denominator = interaction + favs + (smoothingParam / prior)
          UserLabel(
            userId,
            labelName,
            interaction.toInt,
            (interaction + favs).toInt,
            Some(numerator / denominator))
      }
  }

  def getUserLabels(dateRange: DateRange)(implicit tz: TimeZone): Execution[Seq[LabelledUsers]] = {
    val smoothingParam = 0.1
    val favSizes = getLabelSizes(readOutOfNetworkFavs(dateRange)).forceToDiskExecution

    val blockSizes = getLabelSizes(getBlocks(dateRange)).forceToDiskExecution

    val longerDateRange = DateRange(dateRange.end - Days(180), dateRange.end)
    val reportSizes = getLabelSizes(getReports(longerDateRange)).forceToDiskExecution
    val spamReportSizes = getLabelSizes(
      getSpamReports(dateRange, legitOnly = true)).forceToDiskExecution
    val allSpamReportSizes = getLabelSizes(
      getSpamReports(dateRange, legitOnly = false)).forceToDiskExecution

    Execution
      .zip(
        favSizes,
        blockSizes,
        reportSizes,
        spamReportSizes,
        allSpamReportSizes,
      ).map {
        case (favSizes, blockSizes, reportSizes, spamReportSizes, allSpamReportSizes) =>
          val blockScores = computeScoresForInteractions(
            blockSizes,
            favSizes,
            smoothingParam,
            BlocksPerFav.labelName)
          val reportScores = computeScoresForInteractions(
            reportSizes,
            favSizes,
            smoothingParam,
            ReportsPerFav.labelName)
          val spamReportScores = computeScoresForInteractions(
            spamReportSizes,
            favSizes,
            smoothingParam,
            SpamReportsPerFav.labelName)
          val allSpamReportScores = computeScoresForInteractions(
            allSpamReportSizes,
            favSizes,
            smoothingParam,
            AllSpamReportsPerFav.labelName)

          Seq(
            LabelledUsers(blockScores, BlocksPerFav.labelName, BlocksPerFavUserLabelScalaDataset),
            LabelledUsers(
              reportScores,
              ReportsPerFav.labelName,
              ReportsPerFavUserLabelScalaDataset),
            LabelledUsers(
              spamReportScores,
              SpamReportsPerFav.labelName,
              SpamReportsPerFavUserLabelScalaDataset),
            LabelledUsers(
              allSpamReportScores,
              AllSpamReportsPerFav.labelName,
              AllSpamReportsPerFavUserLabelScalaDataset),
          )
      }
  }

  def getBlocks(dateRange: DateRange): TypedPipe[(Long, Long)] = {
    DAL
      .readMostRecentSnapshot(
        FlockBlocksJavaDataset,
        dateRange
      )
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .flatMap { edge =>
        val (sourceId, destId) = (edge.getSourceId, edge.getDestinationId)
        if (edge.getUpdatedAt > (dateRange.start.timestamp / 1000) &&
          edge.getUpdatedAt <= (dateRange.end.timestamp / 1000) &&
          sourceId > 0L && destId > 0L &&
          !ArtificialUserFilter.isKnownArtificialUserId(sourceId) &&
          !ArtificialUserFilter.isKnownArtificialUserId(destId) &&
          ((edge.getStateId == 0) || (edge.getStateId == 2)))
          Some((destId, sourceId))
        else None
      }
      .distinct
  }

  def getReports(dateRange: DateRange)(implicit tz: TimeZone): TypedPipe[(Long, Long)] = {
    DAL
      .read(
        RtpReportsScalaDataset,
        dateRange
      )
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .flatMap { report =>
        for {
          tool <- report.tool
          if tool == Tool.AbuseTriageTool
          reporter <- report.reporterId
          if report.createTimestampInMilliseconds.isDefined
        } yield (report.reportedUserId, reporter)
      }
      .distinct
  }

  def getSpamReports(
    dateRange: DateRange,
    legitOnly: Boolean
  )(
    implicit tz: TimeZone
  ): TypedPipe[(Long, Long)] = {
    val reports: Set[TattleType] = Set(
      TattleType.DirectMessageReport,
      TattleType.ReportAsSpam,
      TattleType.UserReport,
      TattleType.TweetReport,
    )
    DAL
      .read(
        TattleEventsScalaDataset,
        dateRange
      )
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .filter(tattle => !legitOnly || tattle.isLegit.getOrElse(false))
      .filter(tattle => reports.contains(tattle.tattleType))
      .map(tattle => (tattle.spammerId, tattle.victimId))
      .distinct
  }

  def getMentionFilteredEngagements(
    engagementTypeFilter: Set[EngagementType]
  )(
    dateRange: DateRange
  ): TypedPipe[(Long, Long)] = {
    DAL
      .read(MentionInteractionsScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .flatMap { mi =>
        for {
          interactions <- mi.interactions
          hasFilteredEngagement = interactions.userEngagements.exists(e =>
            engagementTypeFilter.contains(e.engagement.engagementType))
          if hasFilteredEngagement
        } yield {
          (mi.mention.tweeterId, mi.mention.mentionedUserId)
        }
      }
      .distinct
  }

  val blocksMatch: Set[EngagementType] = Set(
    EngagementType.Block,
    EngagementType.Mute,
  )

  val reportsMatch: Set[EngagementType] = Set(
    EngagementType.AnnoyingReport,
    EngagementType.HarassmentReport,
    EngagementType.HatefulConductReport,
    EngagementType.ImpersonationReport,
    EngagementType.OffensiveReport,
    EngagementType.PrivateInfoReport,
    EngagementType.SelfHarmReport,
    EngagementType.SpamReport,
    EngagementType.UnauthorizedPhotoReport,
    EngagementType.ViolentThreatReport,
  )

  def getMentionFilteredBlocks: DateRange => TypedPipe[(Long, Long)] =
    getMentionFilteredEngagements(blocksMatch)
  def getMentionFilteredReports: DateRange => TypedPipe[(Long, Long)] =
    getMentionFilteredEngagements(reportsMatch)
}
