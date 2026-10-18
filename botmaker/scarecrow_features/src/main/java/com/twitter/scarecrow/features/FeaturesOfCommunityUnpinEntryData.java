package com.twitter.scarecrow.features;

import java.util.List;
import java.util.stream.Collectors;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.communities.events.thriftjava.CommunityEvent;
import com.twitter.communities.events.thriftjava.CommunityUnpinEntryData;


import static com.twitter.botmaker.FeatureModifier.REQUIRED;
import static com.twitter.botmaker.FeatureModifier.OPTIONAL;

public class FeaturesOfCommunityUnpinEntryData extends FeatureMapExtractor {

  private final CommunityUnpinEntryData event;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfCommunityUnpinEntryData(CommunityUnpinEntryData event,
                                          CommunityEvent baseEvent) {
    this.event = event;
    this.baseEventExtractor = new FeaturesOfCommunityEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    builder.putValue(REQUIRED, BotMakerFeatures.newUnpinnedEntry,
        event.getNewUnpinnedEntry().getEntryId().getTweetId());
    if (event.isSetOldPinnedEntries()) {
        List<Long> oldPinnedEntries = event.getOldPinnedEntries().stream().map(
          entry -> entry.getEntryId().getTweetId()).collect(Collectors.toList());
        builder.putCollection(OPTIONAL, BotMakerFeatures.oldPinnedEntries, oldPinnedEntries,
            Long.class);
    }
  }
}
