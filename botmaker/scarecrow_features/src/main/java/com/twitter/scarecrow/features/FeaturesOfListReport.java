package com.twitter.scarecrow.features;

import com.twitter.reportflow.thriftjava.InAppReport;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfListReport extends FeatureMapExtractor {

  private InAppReport inAppReport;

  public FeaturesOfListReport(InAppReport inAppReportEvent) {
    this.inAppReport = inAppReportEvent;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {

    new FeaturesOfInAppReport(inAppReport).apply(builder);

    builder
        .putValue(REQUIRED, BotMakerFeatures.listId,
            inAppReport.getReportedEntityId().getListId());

  }
}
