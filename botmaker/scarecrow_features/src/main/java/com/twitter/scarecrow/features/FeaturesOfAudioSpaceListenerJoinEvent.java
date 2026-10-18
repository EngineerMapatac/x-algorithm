package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.ListenerJoinEvent;

public class FeaturesOfAudioSpaceListenerJoinEvent extends FeatureMapExtractor {

  private final ListenerJoinEvent listenerJoinEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfAudioSpaceListenerJoinEvent(ListenerJoinEvent listenerJoinEvent,
                                               AudioSpaceBaseEvent baseEvent) {
    this.listenerJoinEvent = listenerJoinEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (listenerJoinEvent.isSetUser_id()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userId,
          listenerJoinEvent.getUser_id());
    }
  }
}
