package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.chubbysnipe.thriftjava.FilteredAuthRequest;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

public class FeaturesOfFilteredAuthRequest extends FeatureMapExtractor {
  private final FilteredAuthRequest event;

  public FeaturesOfFilteredAuthRequest(FilteredAuthRequest event) {
    this.event = event;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.timestampMs,
        event.getTimestampMs());
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.userId, event.getUserId());
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.spammerId, event.getUserId());
    if (event.isSetSessionHash()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.sessionHash,
          event.getSessionHash());
    }
    if (event.isSetClientApplicationId()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.applicationId,
          event.getClientApplicationId());
    }
    if (event.isSetIpAddress()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.ipAddress, event.getIpAddress());
    }
    if (event.isSetUserAgent()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userAgent, event.getUserAgent());
    }
    if (event.isSetCountryCode()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.requestCountryCode,
          event.getCountryCode());
    }
    if (event.isSetAuthenticatedUserId()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.authenticatedUserId,
          event.getAuthenticatedUserId());
    }
    if (event.isSetClient_uuid()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.clientUuid,
          event.getClient_uuid());
    }
  }
}
