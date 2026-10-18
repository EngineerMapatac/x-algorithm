
package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.strato.columns.content_understanding.thriftjava.UnifiedPostAnnotations;
import com.twitter.strato.columns.content_understanding.thriftjava.TweetBoolMetadata;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

public class FeaturesOfUnifiedPostAnnotationsEvent extends FeatureMapExtractor {
    private final UnifiedPostAnnotations event;

    public FeaturesOfUnifiedPostAnnotationsEvent(UnifiedPostAnnotations event) {
        this.event = event;
    }

    @Override
    public void apply(FeatureMapBuilder builder) throws Exception {
        builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.tweetId, event.getTweetId());
        if (event.isSetTweetBoolMetadata()) {
            TweetBoolMetadata tweetBoolMetadata = event.getTweetBoolMetadata();
            if (tweetBoolMetadata.isSetIsHighQuality()) {
                builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isHighQuality,
                        tweetBoolMetadata.getFieldValue(TweetBoolMetadata._Fields.IS_HIGH_QUALITY));
            }
            if (tweetBoolMetadata.isSetIsNsfw()) {
                builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isNsfw,
                        tweetBoolMetadata.getFieldValue(TweetBoolMetadata._Fields.IS_NSFW));
            }
            if (tweetBoolMetadata.isSetIsGore()) {
                builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isGore,
                        tweetBoolMetadata.getFieldValue(TweetBoolMetadata._Fields.IS_GORE));
            }
            if (tweetBoolMetadata.isSetIsViolent()) {
                builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isViolent,
                        tweetBoolMetadata.getFieldValue(TweetBoolMetadata._Fields.IS_VIOLENT));
            }
            if (tweetBoolMetadata.isSetIsSpam()) {
                builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isSpam,
                        tweetBoolMetadata.getFieldValue(TweetBoolMetadata._Fields.IS_SPAM));
            }
            if (tweetBoolMetadata.isSetIsOcr()) {
                builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isOcr,
                        tweetBoolMetadata.getFieldValue(TweetBoolMetadata._Fields.IS_OCR));
            }
            if (tweetBoolMetadata.isSetIsSoftNsfw()) {
                builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isSoftNsfw,
                        tweetBoolMetadata.getFieldValue(TweetBoolMetadata._Fields.IS_SOFT_NSFW));
            }
            if (tweetBoolMetadata.isSetIsAdult()) {
                builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isAdult,
                        tweetBoolMetadata.getFieldValue(TweetBoolMetadata._Fields.IS_ADULT));
            }
        }
        if (event.isSetDescription()) {
            builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.description,
                    event.getDescription());
        }
        if (event.isSetHasImage()) {
            builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.hasImage,
                    event.getFieldValue(UnifiedPostAnnotations._Fields.HAS_IMAGE));
        }
        if (event.isSetHasVideo()) {
            builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.hasVideo,
                    event.getFieldValue(UnifiedPostAnnotations._Fields.HAS_VIDEO));
        }
        if (event.isSetHasMinorScore()) {
            builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.hasMinorScore,
                    event.getHasMinorScore());
        }
    }
}
