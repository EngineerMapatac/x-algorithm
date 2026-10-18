package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.SpaceEndEvent;

public class FeaturesOfAudioSpaceEndEvent extends FeatureMapExtractor {

  private final SpaceEndEvent spaceEndEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfAudioSpaceEndEvent(SpaceEndEvent spaceEndEvent,
                                      AudioSpaceBaseEvent baseEvent) {
    this.spaceEndEvent = spaceEndEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (spaceEndEvent.isSetUser_id()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userId,
          spaceEndEvent.getUser_id());
    }
  }
}
