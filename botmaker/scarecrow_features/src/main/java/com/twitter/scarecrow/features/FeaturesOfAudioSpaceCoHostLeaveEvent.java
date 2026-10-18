package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.CoHostLeaveEvent;

public class FeaturesOfAudioSpaceCoHostLeaveEvent extends FeatureMapExtractor {

  private final CoHostLeaveEvent coHostLeaveEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfAudioSpaceCoHostLeaveEvent(CoHostLeaveEvent coHostLeaveEvent,
                                              AudioSpaceBaseEvent baseEvent) {
    this.coHostLeaveEvent = coHostLeaveEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (coHostLeaveEvent.isSetUser_id()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userId,
          coHostLeaveEvent.getUser_id());
    }
  }
}
