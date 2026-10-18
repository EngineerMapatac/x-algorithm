package com.twitter.scarecrow.features;

import java.util.Map;
import java.util.Set;

import com.google.common.base.Optional;
import com.google.common.base.Preconditions;
import com.google.common.collect.ImmutableSet;

import com.twitter.relevance.feature_store.thrift.FeatureData;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;

public class FeaturesOfTwitterContext extends FeatureMapExtractor {
  private final Optional<Map<String, FeatureData>> allContextFeaturesOpt;

  private static final Set<String> DEFAULT_FEATURES_WANTED = ImmutableSet.of(
      BotMakerFeatures.authenticatedUserId,
      BotMakerFeatures.actorId,
      BotMakerFeatures.ipTags,
      BotMakerFeatures.sessionHash
  );

  public FeaturesOfTwitterContext(Optional<Map<String, FeatureData>> allContextFeaturesOpt) {
    this.allContextFeaturesOpt = Preconditions.checkNotNull(allContextFeaturesOpt);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    apply(builder, DEFAULT_FEATURES_WANTED);
  }

  public void apply(
      FeatureMapBuilder builder,
      Set<String> featuresWanted
  ) throws Exception {
    if (!allContextFeaturesOpt.isPresent()) {
      return;
    }

    Map<String, FeatureData> allFeatures = allContextFeaturesOpt.get();
    for (String feature: featuresWanted) {
      if (allFeatures.containsKey(feature)) {
        builder.putValue(OPTIONAL, feature, allFeatures.get(feature));
      }
    }
  }

  public void applyAll(FeatureMapBuilder builder) throws Exception {
    if (!allContextFeaturesOpt.isPresent()) {
      return;
    }

    Map<String, FeatureData> allFeatures = allContextFeaturesOpt.get();
    for (Map.Entry<String, FeatureData> entry : allFeatures.entrySet()) {
      builder.putValue(OPTIONAL, entry.getKey(), entry.getValue());
    }
  }
}
