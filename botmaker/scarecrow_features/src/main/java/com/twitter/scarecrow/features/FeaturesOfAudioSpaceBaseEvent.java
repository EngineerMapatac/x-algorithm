package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;

public class FeaturesOfAudioSpaceBaseEvent extends FeatureMapExtractor {

  private final AudioSpaceBaseEvent event;

  public FeaturesOfAudioSpaceBaseEvent(AudioSpaceBaseEvent event) {
    this.event = event;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.broadcastId,
        event.getBroadcast_id());
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.timestampMs,
        event.getTime_stamp_millis());
  }
}
