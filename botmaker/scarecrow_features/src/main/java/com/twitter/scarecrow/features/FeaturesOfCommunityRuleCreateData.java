package com.twitter.scarecrow.features;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.communities.events.thriftjava.CommunityEvent;
import com.twitter.communities.events.thriftjava.CommunityRuleCreateData;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;
import static com.twitter.botmaker.FeatureModifier.OPTIONAL;

public class FeaturesOfCommunityRuleCreateData extends FeatureMapExtractor {

  private final CommunityRuleCreateData event;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfCommunityRuleCreateData(CommunityRuleCreateData event,
                                             CommunityEvent baseEvent) {
    this.event = event;
    this.baseEventExtractor = new FeaturesOfCommunityEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    builder.putValue(REQUIRED, BotMakerFeatures.ruleId,
        event.getRuleId());
    builder.putValue(REQUIRED, BotMakerFeatures.name,
        event.getName());
    builder.putValue(REQUIRED, BotMakerFeatures.description,
        event.getDesc());
    builder.putCollection(OPTIONAL, BotMakerFeatures.updatedRuleIds,
        event.getUpdatedRuleIds(), Long.class);
  }
}
