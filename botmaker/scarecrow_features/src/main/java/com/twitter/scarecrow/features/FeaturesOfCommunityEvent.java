package com.twitter.scarecrow.features;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.communities.events.thriftjava.CommunityEvent;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;
import static com.twitter.botmaker.FeatureModifier.OPTIONAL;

public class FeaturesOfCommunityEvent extends FeatureMapExtractor {

  private final CommunityEvent event;

  public FeaturesOfCommunityEvent(CommunityEvent event) {
    this.event = event;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(REQUIRED, BotMakerFeatures.communityId,
        event.getCommunityId());
    builder.putValue(REQUIRED, BotMakerFeatures.eventTimestampMs,
        event.getTimestamp());
    if (event.getCallContext().isSetTwitterUserId()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.userId,
        event.getCallContext().getTwitterUserId());
    }
  }
}
