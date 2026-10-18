package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.SpacePublishEvent;

public class FeaturesOfAudioSpacePublishEvent extends FeatureMapExtractor {

  private final SpacePublishEvent spacePublishEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfAudioSpacePublishEvent(SpacePublishEvent spacePublishEvent,
                                          AudioSpaceBaseEvent baseEvent) {
    this.spacePublishEvent = spacePublishEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (spacePublishEvent.isSetUser_id()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userId,
          spacePublishEvent.getUser_id());
    }
  }
}
