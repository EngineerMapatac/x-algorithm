package com.twitter.scarecrow.features;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceTranscriptionEvent;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;

public class FeaturesOfAudioSpaceTranscriptionEvent extends FeatureMapExtractor {

  private final AudioSpaceTranscriptionEvent event;

  public FeaturesOfAudioSpaceTranscriptionEvent(AudioSpaceTranscriptionEvent event) {
    this.event = event;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(OPTIONAL, BotMakerFeatures.broadcastId,
        event.getBroadcast_id());
    builder.putValue(OPTIONAL, BotMakerFeatures.timestampMs,
        event.getTime_stamp_millis());
    builder.putValue(OPTIONAL, BotMakerFeatures.language,
        event.getBroadcast_language());
    builder.putValue(OPTIONAL, BotMakerFeatures.userId,
        event.getUser_twitter_id());
    builder.putValue(OPTIONAL, BotMakerFeatures.utterance,
        event.getUtterance());
    builder.putValue(OPTIONAL, BotMakerFeatures.chatMessageUuid,
        event.getChat_message_uuid());
  }
}
