package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.guano.thriftjava.DestroyStatus;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfDestroyStatus extends FeatureMapExtractor {
  private final DestroyStatus destroyStatus;

  public FeaturesOfDestroyStatus(DestroyStatus destroyStatus) {
    this.destroyStatus = Preconditions.checkNotNull(destroyStatus);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder
        .putPositive(REQUIRED, BotMakerFeatures.timestampMs, destroyStatus.getTimestamp() * 1000L)
        .putPositive(REQUIRED, BotMakerFeatures.eventId, destroyStatus.getStatusId())
        .putPositive(REQUIRED, BotMakerFeatures.tweetId, destroyStatus.getStatusId())
        .putValue(REQUIRED, BotMakerFeatures.tweetText, destroyStatus.getText())
        .putPositive(REQUIRED, BotMakerFeatures.userId, destroyStatus.getUserId())
        .putPositive(REQUIRED, BotMakerFeatures.spammerId, destroyStatus.getUserId())
        .putPositive(REQUIRED, BotMakerFeatures.byUserId, destroyStatus.getByUserId())
        .putValue(OPTIONAL, BotMakerFeatures.runId, destroyStatus.getRunId())
        .putValue(OPTIONAL, BotMakerFeatures.note, destroyStatus.getNote())
        .putValue(OPTIONAL, BotMakerFeatures.actorName, destroyStatus.getBulkId())
        .putValue(OPTIONAL, BotMakerFeatures.reason, destroyStatus.getReason().name())
        .putValue(OPTIONAL, BotMakerFeatures.contentType, destroyStatus.getType().name());
  }
}
