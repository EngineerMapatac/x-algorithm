package com.twitter.scarecrow.features;

import com.twitter.health.account_integrity.arkose.thriftjava.ArkoseVerify;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.EnumExtractors;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfArkoseVerifyEvent extends FeatureMapExtractor {

    private final ArkoseVerify event;

    public FeaturesOfArkoseVerifyEvent(ArkoseVerify event) {
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
    }
}
