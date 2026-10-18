package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.ListenerLeaveEvent;

public class FeaturesOfAudioSpaceListenerLeaveEvent extends FeatureMapExtractor {

  private final ListenerLeaveEvent listenerLeaveEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfAudioSpaceListenerLeaveEvent(ListenerLeaveEvent listenerLeaveEvent,
                                                AudioSpaceBaseEvent baseEvent) {
    this.listenerLeaveEvent = listenerLeaveEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (listenerLeaveEvent.isSetUser_id()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userId,
          listenerLeaveEvent.getUser_id());
    }
  }
}
