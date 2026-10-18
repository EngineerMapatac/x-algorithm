package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.TitleUpdatedEvent;

public class FeaturesOfTitleUpdatedEvent extends FeatureMapExtractor {

  private final TitleUpdatedEvent titleUpdatedEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfTitleUpdatedEvent(TitleUpdatedEvent titleUpdatedEvent,
                                     AudioSpaceBaseEvent baseEvent) {
    this.titleUpdatedEvent = titleUpdatedEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.title,
        titleUpdatedEvent.getNew_title());
  }
}
