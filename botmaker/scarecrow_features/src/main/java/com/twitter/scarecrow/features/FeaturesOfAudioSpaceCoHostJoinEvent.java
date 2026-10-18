package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.CoHostJoinEvent;

public class FeaturesOfAudioSpaceCoHostJoinEvent extends FeatureMapExtractor {

  private final CoHostJoinEvent coHostJoinEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfAudioSpaceCoHostJoinEvent(CoHostJoinEvent coHostJoinEvent,
                                             AudioSpaceBaseEvent baseEvent) {
    this.coHostJoinEvent = coHostJoinEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (coHostJoinEvent.isSetUser_id()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userId,
          coHostJoinEvent.getUser_id());
    }
  }
}
