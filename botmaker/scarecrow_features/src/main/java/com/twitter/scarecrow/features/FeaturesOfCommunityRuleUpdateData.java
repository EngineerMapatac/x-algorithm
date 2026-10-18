package com.twitter.scarecrow.features;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.communities.events.thriftjava.CommunityEvent;
import com.twitter.communities.events.thriftjava.CommunityRuleUpdateData;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;
import static com.twitter.botmaker.FeatureModifier.OPTIONAL;

public class FeaturesOfCommunityRuleUpdateData extends FeatureMapExtractor {

  private final CommunityRuleUpdateData event;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfCommunityRuleUpdateData(CommunityRuleUpdateData event,
                                             CommunityEvent baseEvent) {
    this.event = event;
    this.baseEventExtractor = new FeaturesOfCommunityEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    builder.putValue(REQUIRED, BotMakerFeatures.ruleId,
        event.getRuleId());
    if (event.isSetOldName()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.oldName,
          event.getOldName());
    }
    if (event.isSetNewName()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.newName,
          event.getNewName());
    }
    if (event.isSetOldDesc()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.oldDesc,
          event.getOldDesc());
    }
    if (event.isSetNewDesc()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.newDesc,
          event.getNewDesc());
    }
  }
}
