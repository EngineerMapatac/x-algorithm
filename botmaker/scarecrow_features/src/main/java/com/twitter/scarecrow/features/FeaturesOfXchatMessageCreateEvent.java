package com.twitter.scarecrow.features;

import java.util.Optional;

import com.google.common.base.Preconditions;
import com.x.dmv2.thriftjava.MessageEvent;
import com.x.dmv2.thriftjava.MessageCreateEvent;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;

public class FeaturesOfXchatMessageCreateEvent extends FeatureMapExtractor {

  private final MessageEvent messageEvent;
  private final MessageCreateEvent messageCreateEvent;

  public FeaturesOfXchatMessageCreateEvent(
      MessageEvent messageEvent,
      MessageCreateEvent messageCreateEvent
  ) {
    this.messageEvent = Preconditions.checkNotNull(messageEvent);
    this.messageCreateEvent = Preconditions.checkNotNull(messageCreateEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    new FeaturesOfXchatMessageEvent(messageEvent).apply(builder);

    Optional<Integer> contentLengthOpt =
        Optional.ofNullable(messageCreateEvent.getContents().length);
    if (contentLengthOpt.isPresent()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.contentLength, contentLengthOpt.get());
    }

    Optional<String> conversationKeyVersion =
        Optional.ofNullable(messageCreateEvent.getConversation_key_version());
    if (conversationKeyVersion.isPresent()) {
      builder.putValue(
          OPTIONAL,
          BotMakerFeatures.conversationKeyVersion,
          conversationKeyVersion.get()
      );
    }

    Optional<Long> ttlMsecOpt = Optional.ofNullable(messageCreateEvent.getTtl_msec());
    if (ttlMsecOpt.isPresent()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.ttlMsec, ttlMsecOpt.get());
    }
  }
}
