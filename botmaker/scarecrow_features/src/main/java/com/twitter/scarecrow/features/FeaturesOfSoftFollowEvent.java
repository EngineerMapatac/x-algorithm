package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.socialgraph.thriftjava.SoftFollowEvent;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfSoftFollowEvent extends FeatureMapExtractor {

  private final SoftFollowEvent softFollowEvent;
  private final FeaturesOfTwitterContext twitterContextFeatures;

  public FeaturesOfSoftFollowEvent(
      SoftFollowEvent softFollowEvent,
      FeaturesOfTwitterContext twitterContextFeatures
  ) {
    this.softFollowEvent = Preconditions.checkNotNull(softFollowEvent);
    this.twitterContextFeatures = Preconditions.checkNotNull(twitterContextFeatures);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {

    builder.putValue(REQUIRED, BotMakerFeatures.sourceId, softFollowEvent.getSource())
        .putValue(REQUIRED, BotMakerFeatures.targetId, softFollowEvent.getTarget())
        .putValue(REQUIRED, BotMakerFeatures.spammerId, softFollowEvent.getSource())
        .putValue(REQUIRED, BotMakerFeatures.victimId, softFollowEvent.getTarget())
        .putValue(REQUIRED, BotMakerFeatures.eventTimestampMs,
            softFollowEvent.getTimestamp());

    twitterContextFeatures.applyAll(builder);
  }
}
