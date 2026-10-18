package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.tweetypie.thriftjava.TweetDeleteEvent;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfTweetDelete extends FeatureMapExtractor {

  private final TweetDeleteEvent tweetDelete;
  private final long timestampMs;
  private final FeaturesOfTwitterContext twitterContextFeatures;

  public FeaturesOfTweetDelete(
      TweetDeleteEvent tweetDelete,
      long timestampMs,
      FeaturesOfTwitterContext context
  ) {
    this.tweetDelete = Preconditions.checkNotNull(tweetDelete);
    this.timestampMs = timestampMs;
    this.twitterContextFeatures = Preconditions.checkNotNull(context);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(REQUIRED, BotMakerFeatures.eventTimestampMs, timestampMs);

    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isErasure,
        tweetDelete.is_user_erasure);

    new FeaturesOfTweet(tweetDelete.tweet, false).apply(builder);

    twitterContextFeatures.applyAll(builder);
  }
}
