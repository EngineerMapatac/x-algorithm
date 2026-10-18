package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.tweetypie.thriftjava.Tweet;
import com.twitter.tweetypie.thriftjava.TweetUndeleteEvent;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfTweetUndelete extends FeatureMapExtractor {
  private final TweetUndeleteEvent tweetUndelete;
  private final long timestampMs;
  private final boolean isRetweet;
  private final boolean existsQuotedTweet;
  private final FeaturesOfTwitterContext twitterContextFeatures;

  public FeaturesOfTweetUndelete(
      TweetUndeleteEvent tweetUndelete,
      long timestampMs,
      boolean isRetweet,
      boolean existsQuotedTweet,
      FeaturesOfTwitterContext context
  ) {
    this.tweetUndelete = Preconditions.checkNotNull(tweetUndelete);
    this.timestampMs = timestampMs;
    this.isRetweet = isRetweet;
    this.existsQuotedTweet = existsQuotedTweet;
    this.twitterContextFeatures = Preconditions.checkNotNull(context);
  }
  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(REQUIRED, BotMakerFeatures.eventTimestampMs, timestampMs);

    new FeaturesOfTweet(tweetUndelete.tweet, true).apply(builder);

    if (isRetweet) {
      builder.putValue(REQUIRED, BotMakerFeatures.victimId,
          tweetUndelete.getRetweet_parent_user_id());
    }

    if (existsQuotedTweet) {
      final Tweet quotedTweet = tweetUndelete.getQuoted_tweet();
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
