package com.twitter.scarecrow.features;

import java.util.Map;

import com.twitter.health.gbe.async.Mapper;
import com.twitter.health.thriftscala.HealthGenericEntityEvent;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfHealthGenericEntityEvent extends FeatureMapExtractor {

  private final HealthGenericEntityEvent healthGenericEntityEvent;

  public FeaturesOfHealthGenericEntityEvent(
      final HealthGenericEntityEvent healthGenericEntityEvent) {
    this.healthGenericEntityEvent = healthGenericEntityEvent;
  }

  @Override
  public void apply(final FeatureMapBuilder featureMapBuilder) throws Exception {
    Map<String, Object> map = Mapper.applyJava(healthGenericEntityEvent);

    for (Map.Entry<String, Object> e: map.entrySet()) {
      featureMapBuilder.putValue(REQUIRED, e.getKey(), e.getValue());
    }
  }
}
