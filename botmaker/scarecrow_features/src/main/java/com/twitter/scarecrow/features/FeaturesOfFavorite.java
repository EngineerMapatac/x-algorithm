package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.timelineservice.thriftjava.FavoriteEvent;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;
import static com.twitter.spam.botmaker_features.FeatureExtractionHelper.getLongFeatureData;

public class FeaturesOfFavorite extends FeatureMapExtractor {
  private final FavoriteEvent favoriteEvent;
  private final FeaturesOfTwitterContext twitterContextFeatures;

  public FeaturesOfFavorite(
      FavoriteEvent favoriteEvent,
      FeaturesOfTwitterContext twitterContextFeatures
  ) {
    this.favoriteEvent = Preconditions.checkNotNull(favoriteEvent);
    this.twitterContextFeatures = Preconditions.checkNotNull(twitterContextFeatures);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder
        .putValue(REQUIRED, BotMakerFeatures.eventTimestampMs,
            getLongFeatureData(favoriteEvent.getEvent_time_ms()))
        .putValue(REQUIRED, BotMakerFeatures.actorId,
            getLongFeatureData(favoriteEvent.getUser_id()))
        .putValue(REQUIRED, BotMakerFeatures.spammerId,
            getLongFeatureData(favoriteEvent.getUser_id()))
        .putValue(REQUIRED, BotMakerFeatures.victimId,
            getLongFeatureData(favoriteEvent.getTweet_user_id()))
        .putValue(REQUIRED, BotMakerFeatures.tweetId,
            getLongFeatureData(favoriteEvent.getTweet_id()));

    twitterContextFeatures.applyAll(builder);
  }
}
