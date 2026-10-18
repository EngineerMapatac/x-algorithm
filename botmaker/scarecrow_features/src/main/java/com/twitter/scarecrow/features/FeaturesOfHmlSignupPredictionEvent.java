package com.twitter.scarecrow.features;

import com.twitter.abuse.detection.dataflow.prediction.thriftjava.HmlSignupPredictionEvent;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfHmlSignupPredictionEvent extends FeatureMapExtractor {
  private final HmlSignupPredictionEvent event;

  public FeaturesOfHmlSignupPredictionEvent(HmlSignupPredictionEvent event) {
    this.event = event;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(REQUIRED, BotMakerFeatures.userId, event.getUser_id());
    builder.putValue(REQUIRED, BotMakerFeatures.timestampMs, event.getUc10_timestamp_ms());
    builder.putValue(REQUIRED, BotMakerFeatures.predictTimestampMs,
        event.getPredict_timestamp_ms());
    builder.putValue(REQUIRED, BotMakerFeatures.scoreMap, event.getScore_by_model_name());
  }
}
