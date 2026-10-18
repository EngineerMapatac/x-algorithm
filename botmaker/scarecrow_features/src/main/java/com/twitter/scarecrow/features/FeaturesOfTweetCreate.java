package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.tweetypie.thriftjava.Tweet;
import com.twitter.tweetypie.thriftjava.TweetCreateEvent;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfTweetCreate extends FeatureMapExtractor {

  private final TweetCreateEvent tweetCreate;
  private final long timestampMs;
  private final boolean isTweetEvent;
  private final boolean isRetweet;
  private final boolean existsQuotedTweet;
  private final FeaturesOfTwitterContext twitterContextFeatures;

  public FeaturesOfTweetCreate(
      TweetCreateEvent tweetCreate,
      long timestampMs,
      boolean isTweetEvent,
      boolean isRetweet,
      boolean existsQuotedTweet,
      FeaturesOfTwitterContext context
  ) {
    this.tweetCreate = Preconditions.checkNotNull(tweetCreate);
    this.timestampMs = timestampMs;
    this.isTweetEvent = isTweetEvent;
    this.isRetweet = isRetweet;
    this.existsQuotedTweet = existsQuotedTweet;
    this.twitterContextFeatures = Preconditions.checkNotNull(context);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {

    new FeaturesOfTweet(tweetCreate.tweet, isTweetEvent).apply(builder);

    builder.putValue(REQUIRED, BotMakerFeatures.eventTimestampMs, timestampMs);

    if (isRetweet) {
      builder.putValue(REQUIRED, BotMakerFeatures.victimId,
          tweetCreate.getRetweet_parent_user_id());
    }

    if (existsQuotedTweet) {
      final Tweet quotedTweet = tweetCreate.getQuoted_tweet();
      builder.putValue(OPTIONAL, BotMakerFeatures.quotedTweetId,
          quotedTweet.getId());
      if (quotedTweet.isSetCore_data()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.quotedTweetUserId,
            quotedTweet.core_data.getUser_id());
        builder.putValue(OPTIONAL, BotMakerFeatures.quotedTweetText,
            quotedTweet.core_data.getText());
      }
    }

    twitterContextFeatures.applyAll(builder);
  }
}
