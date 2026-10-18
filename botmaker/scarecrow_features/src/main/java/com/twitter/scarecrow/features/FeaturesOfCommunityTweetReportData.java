package com.twitter.scarecrow.features;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.communities.events.thriftjava.CommunityEvent;
import com.twitter.communities.events.thriftjava.CommunityTweetReportData;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;
import static com.twitter.botmaker.FeatureModifier.OPTIONAL;

public class FeaturesOfCommunityTweetReportData extends FeatureMapExtractor {

  private final CommunityTweetReportData event;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfCommunityTweetReportData(CommunityTweetReportData event,
                                             CommunityEvent baseEvent) {
    this.event = event;
    this.baseEventExtractor = new FeaturesOfCommunityEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    builder.putValue(REQUIRED, BotMakerFeatures.tweetId,
        event.getTweetId());
    builder.putValue(REQUIRED, BotMakerFeatures.reporterId,
        event.getReporterId());
    if (event.isSetRuleId()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.ruleId,
        event.getRuleId());
    }
  }
}
