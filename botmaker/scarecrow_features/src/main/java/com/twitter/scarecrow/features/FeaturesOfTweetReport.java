package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.reportflow.thriftjava.InAppReport;
import com.twitter.reportflow.thriftjava.ReportedEntityId;
import com.twitter.reportflow.thriftjava.VictimType;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureExtractionException;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfTweetReport extends FeatureMapExtractor {

  private final InAppReport inAppReport;
  private final FeaturesOfTwitterContext twitterContextFeatures;

  private static final String ME = "Me";
  private static final String COMPANY = "Company";
  private static final String GROUP = "Group";
  private static final String I_REPRESENT = "I_represent";
  private static final String REPORTED_USER = "Reported_user";
  private static final String SOMEONE_ELSE = "Someone_else";

  public FeaturesOfTweetReport(
      InAppReport inAppReportEvent,
      FeaturesOfTwitterContext twitterContextFeatures
  ) {
    this.inAppReport = inAppReportEvent;
    this.twitterContextFeatures = Preconditions.checkNotNull(twitterContextFeatures);
  }

  private String getVictimType(VictimType victimType) {
    if (victimType == null) {
      return "";
    }

    switch(victimType) {
      case REPORTING_USER: return ME;
      case REPORTED_USER: return REPORTED_USER;
      case COMPANY_OF_REPORTING_USER: return COMPANY;
      case REPRESENTATION_OF_REPORTING_USER: return I_REPRESENT;
      case SOMEONE_ELSE: return SOMEONE_ELSE;
      case GROUP: return GROUP;
      default: return "";
    }
  }

  private void processTweetIdentifier(FeatureMapBuilder builder)
      throws FeatureExtractionException {

    long tweetId = 0L;
    if (inAppReport.reportedEntityId.getSetField() == ReportedEntityId._Fields.TWEET_ID) {
      tweetId = inAppReport.getReportedEntityId().getTweetId();
    } else {
      if (inAppReport.isSetEntityReportDetails()
          && inAppReport.getEntityReportDetails().getMomentReportDetails() != null) {
        tweetId = inAppReport.getEntityReportDetails().getMomentReportDetails().tweetId;
      }
    }

    if (tweetId != 0) {
      builder
          .putValue(OPTIONAL, BotMakerFeatures.eventId, tweetId)
          .putValue(OPTIONAL, BotMakerFeatures.sourceEventId, tweetId)
          .putValue(OPTIONAL, BotMakerFeatures.tweetId, tweetId);
    }
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {

    long reporterId = inAppReport.getReporterId();
    long reportedUserId = inAppReport.getReportedUserId();
    String victimType = inAppReport.isSetVictimType()
        ? getVictimType(inAppReport.victimType) : "";

    FeaturesOfInAppReport featuresOfInAppReport = new FeaturesOfInAppReport(inAppReport);
    featuresOfInAppReport.apply(builder);

    builder
        .putValue(REQUIRED, BotMakerFeatures.spammerId, reportedUserId)
        .putValue(REQUIRED, BotMakerFeatures.actorId, reporterId)
        .putValue(REQUIRED, BotMakerFeatures.victimId, reporterId);

    processTweetIdentifier(builder);

    if (inAppReport.reportedEntityId.getSetField() == ReportedEntityId._Fields.MOMENT_ID) {
      builder.putValue(OPTIONAL, BotMakerFeatures.momentId,
          inAppReport.getReportedEntityId().getMomentId());
    }

    if (inAppReport.isSetAdditionalReportedEntities()) {
      featuresOfInAppReport.processAdditionalReportedTweets(builder);
    }

    if (!victimType.isEmpty()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.victimType, victimType);
    }

    twitterContextFeatures.applyAll(builder);
  }
}
