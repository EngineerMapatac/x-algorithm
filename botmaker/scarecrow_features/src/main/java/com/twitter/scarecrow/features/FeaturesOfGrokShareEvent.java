package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.search_router.thriftjava.GrokShareItem;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.EnumExtractors;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

public class FeaturesOfGrokShareEvent extends FeatureMapExtractor {
  private final GrokShareItem event;

  public FeaturesOfGrokShareEvent(GrokShareItem event) {
    this.event = event;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.userId, event.getUserId());
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.conversationId,
        event.getConversationId());
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isConversationShare,
        event.getFieldValue(GrokShareItem._Fields.IS_CONVERSATION_SHARE));
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.rangeStart, event.getRangeStart());
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.rangeEnd, event.getRangeEnd());
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.createdAtMs,
        event.getCreatedAtMsec());
    if (event.isSetGrokShareSource()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.grokShareSource,
          EnumExtractors.name(event.getGrokShareSource()));
    }
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.grokMediaIds, event.getMediaIds());
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.shareId, event.getShareId());
  }
}
