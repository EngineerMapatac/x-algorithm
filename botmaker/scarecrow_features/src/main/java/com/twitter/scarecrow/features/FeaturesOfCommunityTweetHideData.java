package com.twitter.scarecrow.features;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.communities.events.thriftjava.CommunityEvent;
import com.twitter.communities.events.thriftjava.CommunityTweetHideData;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfCommunityTweetHideData extends FeatureMapExtractor {

  private final CommunityTweetHideData event;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfCommunityTweetHideData(CommunityTweetHideData event,
                                             CommunityEvent baseEvent) {
    this.event = event;
    this.baseEventExtractor = new FeaturesOfCommunityEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    builder.putValue(REQUIRED, BotMakerFeatures.tweetId,
        event.getTweetId());
  }
}
