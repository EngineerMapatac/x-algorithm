package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.SpaceScheduledEvent;

public class FeaturesOfAudioSpaceScheduledEvent extends FeatureMapExtractor {

  private final SpaceScheduledEvent spaceScheduledEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfAudioSpaceScheduledEvent(SpaceScheduledEvent spaceScheduledEvent,
                                            AudioSpaceBaseEvent baseEvent) {
    this.spaceScheduledEvent = spaceScheduledEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (spaceScheduledEvent.isSetUser_id()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userId,
          spaceScheduledEvent.getUser_id());
    }
    if (spaceScheduledEvent.isSetScheduled_start_time_unix_millis()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.eventTimestampMs,
          spaceScheduledEvent.getScheduled_start_time_unix_millis());
    }
  }
}
