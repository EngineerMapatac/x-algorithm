package com.twitter.scarecrow.features;

import com.twitter.reportflow.thriftjava.InAppReport;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfDMConversationReport extends FeatureMapExtractor {

  private InAppReport inAppReport;

  public FeaturesOfDMConversationReport(InAppReport inAppReportEvent) {
    this.inAppReport = inAppReportEvent;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {

    FeaturesOfInAppReport featuresOfInAppReport = new FeaturesOfInAppReport(inAppReport);
    featuresOfInAppReport.apply(builder);

    builder
        .putValue(REQUIRED, BotMakerFeatures.dmConversationId,
            inAppReport.getReportedEntityId().getDmConversationId());

    if (inAppReport.isSetAdditionalReportedEntities()) {
      featuresOfInAppReport.processAdditionalReportedDMs(builder);
    }
  }
}
