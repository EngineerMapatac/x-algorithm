package com.twitter.scarecrow.features;

import java.util.ArrayList;
import java.util.List;

import com.twitter.clientapp.gen.Item;
import com.twitter.clientapp.gen.LogEvent;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfClientEventTcoResolution extends FeatureMapExtractor {

  private final LogEvent logEvent;

  public FeaturesOfClientEventTcoResolution(LogEvent logEvent) {
    this.logEvent = logEvent;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(REQUIRED, BotMakerFeatures.tcoString, logEvent.getMessage());
    List<String> resolution = new ArrayList<>();
    for (Item item : logEvent.getEvent_details().getItems()) {
      if (item.isSetRedirect_hop_details()) {
        resolution.add(item.getRedirect_hop_details().getUrl());
      } else {
        resolution.add("");
      }
    }
    builder.putCollection(
        REQUIRED, BotMakerFeatures.clientBasedResolutionChain, resolution, String.class);
    builder.putValue(REQUIRED, BotMakerFeatures.timestampMs, logEvent.getLog_base().getTimestamp());
    if (logEvent.isSetEvent_namespace() && logEvent.getEvent_namespace().isSetPage()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.resolutionPage,
          logEvent.getEvent_namespace().getPage());
    }
  }
}
