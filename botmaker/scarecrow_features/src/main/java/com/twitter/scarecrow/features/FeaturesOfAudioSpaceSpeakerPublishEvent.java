package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.SpeakerPublishEvent;

public class FeaturesOfAudioSpaceSpeakerPublishEvent extends FeatureMapExtractor {

  private final SpeakerPublishEvent speakerPublishEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfAudioSpaceSpeakerPublishEvent(SpeakerPublishEvent speakerPublishEvent,
                                                 AudioSpaceBaseEvent baseEvent) {
    this.speakerPublishEvent = speakerPublishEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (speakerPublishEvent.isSetUser_id()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userId,
          speakerPublishEvent.getUser_id());
    }
    if (speakerPublishEvent.isSetHost_user_id()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.hostUserId,
          speakerPublishEvent.getHost_user_id());
    }
    builder.putCollection(FeatureModifier.OPTIONAL, BotMakerFeatures.adminUserIds,
        speakerPublishEvent.getAdmin_user_ids(), Long.class);
  }
}
