package com.twitter.scarecrow.features;

import com.twitter.reportflow.thriftjava.InAppReport;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfDMReport extends FeatureMapExtractor {

  private InAppReport inAppReport;

  public FeaturesOfDMReport(InAppReport inAppReportEvent) {
    this.inAppReport = inAppReportEvent;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {

    FeaturesOfInAppReport featuresOfInAppReport = new FeaturesOfInAppReport(inAppReport);
    featuresOfInAppReport.apply(builder);

    long dmId = inAppReport.getReportedEntityId().getDmId();
    long reporterId = inAppReport.getReporterId();
    long reportedUserId = inAppReport.getReportedUserId();

    builder
        .putValue(REQUIRED, BotMakerFeatures.dmId, dmId)
        .putValue(REQUIRED, BotMakerFeatures.eventId, dmId)
        .putValue(REQUIRED, BotMakerFeatures.spammerId, reportedUserId)
        .putValue(REQUIRED, BotMakerFeatures.actorId, reporterId)
        .putValue(REQUIRED, BotMakerFeatures.victimId, reporterId);

    if (inAppReport.isSetAdditionalReportedEntities()) {
      featuresOfInAppReport.processAdditionalReportedDMs(builder);
    }
  }
}
