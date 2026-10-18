package com.twitter.scarecrow.features;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.communities.events.thriftjava.CommunityEvent;
import com.twitter.communities.events.thriftjava.CommunityJoinRequestsAutoEnrollData;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;

public class FeaturesOfCommunityJoinRequestsAutoEnrollData extends FeatureMapExtractor {

  private final CommunityJoinRequestsAutoEnrollData event;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfCommunityJoinRequestsAutoEnrollData(CommunityJoinRequestsAutoEnrollData event,
                                                        CommunityEvent baseEvent) {
    this.event = event;
    this.baseEventExtractor = new FeaturesOfCommunityEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (event.isSetNewMembers()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.newMembers,
        event.getNewMembers());
    }
  }
}
