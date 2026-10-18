package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.reportflow.thriftjava.InAppReport;
import com.twitter.reportflow.thriftjava.ReportType;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;
import static com.twitter.botmaker.FeatureModifier.OPTIONAL;


public class FeaturesOfUserReport extends FeatureMapExtractor {

  private InAppReport inAppReport;
  private final FeaturesOfTwitterContext twitterContextFeatures;

  public FeaturesOfUserReport(
      InAppReport inAppReportEvent,
      FeaturesOfTwitterContext twitterContextFeatures
  ) {
    this.inAppReport = inAppReportEvent;
    this.twitterContextFeatures = Preconditions.checkNotNull(twitterContextFeatures);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {

    FeaturesOfInAppReport featuresOfInAppReport = new FeaturesOfInAppReport(inAppReport);
    featuresOfInAppReport.apply(builder);

    long userId = inAppReport.getReportedEntityId().getUserId();
    long reporterId = inAppReport.getReporterId();
    long reportedUserId = inAppReport.getReportedUserId();
    String spamType = inAppReport.getReportType() == ReportType.SPAM
        ? featuresOfInAppReport.getSpamType(inAppReport.getReportFlowName()) : "";
    String abuseType = inAppReport.getReportType() == ReportType.ABUSE
        ? featuresOfInAppReport.getAbuseType(inAppReport.getReportFlowName()) : "";
    String reportedAs =  featuresOfInAppReport.getReportType(inAppReport.getReportType());
    String victimType = inAppReport.isSetVictimType()
        ? featuresOfInAppReport.getVictimTypeV2(inAppReport.getVictimType()) : "";

    builder
        .putValue(REQUIRED,
            BotMakerFeatures.userId, userId)
        .putValue(REQUIRED, BotMakerFeatures.actorId, reporterId)
        .putValue(REQUIRED, BotMakerFeatures.victimId, reporterId)
        .putValue(REQUIRED, BotMakerFeatures.victimIdV2, reporterId)
        .putValue(REQUIRED, BotMakerFeatures.reportedAsV2, reportedAs)
        .putValue(OPTIONAL, BotMakerFeatures.spammerId, reportedUserId);

    if (!spamType.isEmpty()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.spamTypeV2, spamType);
    }

    if (!abuseType.isEmpty()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.abuseTypeV2, abuseType);
    }

    if (!victimType.isEmpty()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.victimType, victimType);
      builder.putValue(OPTIONAL, BotMakerFeatures.victimTypeSnakeCase, victimType);
    }

    if (inAppReport.isSetClientContext()) {
      if (inAppReport.getClientContext().isSetApplicationId()) {
        builder.putValue(OPTIONAL,
            BotMakerFeatures.applicationIdV2, inAppReport.getClientContext().getApplicationId());
      }

      if (inAppReport.getClientContext().isSetUserAgent()) {
        builder.putValue(OPTIONAL,
            BotMakerFeatures.userAgentV2, inAppReport.getClientContext().getUserAgent());
      }
    }

    if (inAppReport.isSetAdditionalReportedEntities()) {
      featuresOfInAppReport.processAdditionalReportedTweets(builder);
    }

    twitterContextFeatures.applyAll(builder);
  }
}
