package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.botmaker.logs.thriftjava.BotSmyterLabel;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

public class FeaturesOfBotSmyterLabel extends FeatureMapExtractor {
  private final BotSmyterLabel event;

  public FeaturesOfBotSmyterLabel(BotSmyterLabel event) {
    this.event = event;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.entityType, event.getEntityType());
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.entityId, event.getEntityId());
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.label, event.getLabel());
    builder.putValue(
        FeatureModifier.OPTIONAL, BotMakerFeatures.manualReview, event.getManualReview());
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.rules, event.getRules());
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.status, event.getStatus());
    builder.putValue(
        FeatureModifier.OPTIONAL, BotMakerFeatures.ruleDescription, event.getRuleDescription());
    builder.putValue(
        FeatureModifier.OPTIONAL, BotMakerFeatures.ruleFeatures, event.getRuleFeatures());
  }
}
