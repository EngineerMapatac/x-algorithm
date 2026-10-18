package com.twitter.scarecrow.features;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.EnumExtractors;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.spam.botmaker_features.TryOrNullExtractor;
import com.twitter.spam.url_reputation.thriftjava.UrlReputationChangeLogEntry;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfUrlReputationChange extends FeatureMapExtractor {

  private final UrlReputationChangeLogEntry urlReputationChange;

  public FeaturesOfUrlReputationChange(UrlReputationChangeLogEntry urlReputationChange) {
    this.urlReputationChange = urlReputationChange;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder
        .putExtractor(REQUIRED, BotMakerFeatures.entity, new TryOrNullExtractor<String>() {
          @Override
          protected String tryApply() throws Exception {
            return urlReputationChange.getKey().getCanonicalizedEntity();
          }
        }).putExtractor(REQUIRED, BotMakerFeatures.matchingType,
            new EnumExtractors.Value<>(urlReputationChange.getKey() == null
                ? null : urlReputationChange.getKey().getMatchingType()))
        .putExtractor(OPTIONAL, BotMakerFeatures.action,
            new EnumExtractors.Name<>(urlReputationChange.getAction()))
        .putValue(OPTIONAL, BotMakerFeatures.source,
            urlReputationChange.getSource())
        .putExtractor(OPTIONAL, BotMakerFeatures.verdict,
            new EnumExtractors.Name<>(urlReputationChange.getVerdict()))
        .putValue(OPTIONAL, BotMakerFeatures.originalEntity,
            urlReputationChange.getOriginalEntity())
        .putPositive(OPTIONAL, BotMakerFeatures.timestampMs,
            urlReputationChange.getTimestampMs())
        .putValue(OPTIONAL, BotMakerFeatures.note,
            urlReputationChange.getNote());
  }
}
