package com.twitter.scarecrow.features;

import com.twitter.timelineservice.thriftjava.BookmarkEvent;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfBookmarkEvent extends FeatureMapExtractor {

    private final BookmarkEvent event;

    public FeaturesOfBookmarkEvent(BookmarkEvent event) {
        this.event = event;
    }

    @Override
    public void apply(FeatureMapBuilder builder) throws Exception {
        builder.putValue(REQUIRED, BotMakerFeatures.userId,
                event.getUser_id());
        builder.putValue(REQUIRED, BotMakerFeatures.tweetId,
                event.getTweet_id());
        builder.putValue(REQUIRED, BotMakerFeatures.timestampMs,
                event.getEvent_time_ms());
    }
}
