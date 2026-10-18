package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.timelineservice.thriftjava.UnfavoriteEvent;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;
import static com.twitter.spam.botmaker_features.FeatureExtractionHelper.getLongFeatureData;

public class FeaturesOfUnfavorite extends FeatureMapExtractor {
  private final UnfavoriteEvent unfavoriteEvent;
  private final FeaturesOfTwitterContext twitterContextFeatures;

  public FeaturesOfUnfavorite(
      UnfavoriteEvent unfavoriteEvent,
      FeaturesOfTwitterContext twitterContextFeatures
  ) {
    this.unfavoriteEvent = Preconditions.checkNotNull(unfavoriteEvent);
    this.twitterContextFeatures = Preconditions.checkNotNull(twitterContextFeatures);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder
        .putValue(REQUIRED, BotMakerFeatures.eventTimestampMs,
            getLongFeatureData(unfavoriteEvent.getEvent_time_ms()))
        .putValue(REQUIRED, BotMakerFeatures.actorId,
            getLongFeatureData(unfavoriteEvent.getUser_id()))
        .putValue(REQUIRED, BotMakerFeatures.spammerId,
            getLongFeatureData(unfavoriteEvent.getUser_id()))
        .putValue(REQUIRED, BotMakerFeatures.victimId,
            getLongFeatureData(unfavoriteEvent.getTweet_user_id()))
        .putValue(REQUIRED, BotMakerFeatures.tweetId,
            getLongFeatureData(unfavoriteEvent.getTweet_id()));

    twitterContextFeatures.applyAll(builder);
  }
}
