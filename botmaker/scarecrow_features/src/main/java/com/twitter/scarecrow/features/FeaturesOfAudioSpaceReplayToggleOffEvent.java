package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.ReplayToggleOffEvent;

public class FeaturesOfAudioSpaceReplayToggleOffEvent extends FeatureMapExtractor {

  private final ReplayToggleOffEvent replayToggleOffEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfAudioSpaceReplayToggleOffEvent(ReplayToggleOffEvent replayToggleOffEvent,
                                                  AudioSpaceBaseEvent baseEvent) {
    this.replayToggleOffEvent = replayToggleOffEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (replayToggleOffEvent.isSetBroadcast_ended()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.broadcastEnded,
          replayToggleOffEvent.broadcast_ended);
    }
  }
}
