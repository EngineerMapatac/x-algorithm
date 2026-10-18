package com.twitter.scarecrow.features;

import com.twitter.health.account_integrity.castle.thriftjava.CastleEvent;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.EnumExtractors;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfCastleEvent extends FeatureMapExtractor {

    private final CastleEvent event;

    public FeaturesOfCastleEvent(CastleEvent event) {
        this.event = event;
    }

    @Override
    public void apply(FeatureMapBuilder builder) throws Exception {
        builder.putValue(OPTIONAL, BotMakerFeatures.userId,
                event.getUser_id());
        builder.putValue(OPTIONAL, BotMakerFeatures.guestId,
                event.getGuest_id());
        builder.putValue(REQUIRED, BotMakerFeatures.timestampMs,
                event.getTimestamp_ms());
        builder.putExtractor(REQUIRED, BotMakerFeatures.eventType,
                new EnumExtractors.Name<>(event.getEvent_type()));
        builder.putExtractor(REQUIRED, BotMakerFeatures.eventStatus,
                new EnumExtractors.Name<>(event.getEvent_status()));
    }
}
