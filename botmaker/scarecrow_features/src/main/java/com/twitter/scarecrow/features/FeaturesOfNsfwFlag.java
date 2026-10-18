package com.twitter.scarecrow.features;

import com.twitter.reportflow.thriftjava.InAppReport;
import com.twitter.reportflow.thriftjava.ReportedEntityId;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureExtractionException;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfNsfwFlag extends FeatureMapExtractor {

  private InAppReport inAppReport;

  public FeaturesOfNsfwFlag(InAppReport inAppReportEvent) {
    this.inAppReport = inAppReportEvent;
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
          .putValue(OPTIONAL, BotMakerFeatures.tweetId, tweetId);
    }
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {

    new FeaturesOfInAppReport(inAppReport).apply(builder);

    processTweetIdentifier(builder);

    long reporterId = inAppReport.getReporterId();
    long reportedUserId = inAppReport.getReportedUserId();

    builder
        .putValue(REQUIRED, BotMakerFeatures.spammerId, reportedUserId)
        .putValue(REQUIRED, BotMakerFeatures.actorId, reportedUserId)
        .putValue(REQUIRED, BotMakerFeatures.victimId, reporterId);
  }
}
