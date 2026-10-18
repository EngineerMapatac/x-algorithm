package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import tv.periscope.thriftjava.Broadcast;
import tv.periscope.thriftjava.EventInfo;
import tv.periscope.thriftjava.PeriscopeEvent;
import tv.periscope.thriftjava.SmyteEvent;
import tv.periscope.thriftjava.User;

public class FeaturesOfPeriscopePublishBroadcastEvent extends FeatureMapExtractor {

  private final PeriscopeEvent event;

  public FeaturesOfPeriscopePublishBroadcastEvent(PeriscopeEvent event) {
    this.event = event;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    if (event.getSetField() == PeriscopeEvent._Fields.SMYTE_EVENT) {
      final SmyteEvent baseEvent = event.getSmyteEvent();
      final EventInfo eventInfo = baseEvent.getEvent();
      if (eventInfo.isSetTarget_broadcast()) {
        final Broadcast broadcast = eventInfo.getTarget_broadcast();
        if (broadcast.isSetId()) {
          builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.broadcastId,
              broadcast.getId());
        }
        if (broadcast.isSetLanguage()) {
          builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.language,
              broadcast.getLanguage());
        }
        if (broadcast.isSetTitle()) {
          builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.title,
              broadcast.getTitle());
        }
        if (broadcast.isSetCreated_timestamp_ms()) {
          builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.createdAtMs,
              broadcast.getCreated_timestamp_ms());
        }
        if (broadcast.isSetBroadcaster()) {
          final User broadcaster = broadcast.getBroadcaster();
          if (broadcaster.isSetTwitter_id()) {
            builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userId,
                broadcaster.getTwitter_id());
          }
          if (broadcaster.isSetUser_id()) {
            builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userPeriscopeId,
                broadcaster.getUser_id());
          }
        }
      }
    }
  }
}
