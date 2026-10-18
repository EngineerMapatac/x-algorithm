package com.twitter.scarecrow.features;

import com.twitter.communities.events.thriftjava.CommunityEvent;
import com.twitter.communities.events.thriftjava.CommunityJoinData;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfCommunityJoinEvent extends FeatureMapExtractor {
    private final CommunityJoinData joinData;
    private final CommunityEvent communityEvent;

    public FeaturesOfCommunityJoinEvent(CommunityJoinData joinData, CommunityEvent communityEvent) {
        this.joinData = joinData;
        this.communityEvent = communityEvent;
    }

    @Override
    public void apply(FeatureMapBuilder builder) throws Exception {
        if (joinData.isSetUserId()) {
            builder.putValue(REQUIRED, BotMakerFeatures.userId, joinData.getUserId());
        }
        builder.putValue(REQUIRED, BotMakerFeatures.communityId,
                communityEvent.getCommunityId());
        builder.putValue(REQUIRED, BotMakerFeatures.eventTimestampMs,
                communityEvent.getTimestamp());
    }
}
