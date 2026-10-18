package com.twitter.scarecrow.features;

import com.twitter.passbird.thrift.event.ClientApplicationEvent;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfClientApplicationEvent extends FeatureMapExtractor {

  private final ClientApplicationEvent event;

  public FeaturesOfClientApplicationEvent(ClientApplicationEvent event) {
    this.event = event;
  }


  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(REQUIRED, BotMakerFeatures.applicationId, event.getClient_application_id());
    builder.putValue(REQUIRED, BotMakerFeatures.eventTimestampMs, event.getTimestamp_ms());

    builder.putValue(OPTIONAL, BotMakerFeatures.name, event.getName());
    builder.putValue(OPTIONAL, BotMakerFeatures.description, event.getDescription());
    builder.putValue(OPTIONAL, BotMakerFeatures.note, event.getNote());
    builder.putValue(OPTIONAL, BotMakerFeatures.url, event.getUrl());
    builder.putValue(OPTIONAL, BotMakerFeatures.callbackUrl, event.getCallback_url());
    builder.putValue(OPTIONAL, BotMakerFeatures.supportUrl, event.getSupport_url());

    if (event.isSetUser_id()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.ownerId, event.getUser_id());
    }
    if (event.isSetCreated_at()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.createdAtMs, event.getCreated_at());
    }
    if (event.isSetSource_user_id()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.actorId, event.getSource_user_id());
    }
  }
}
