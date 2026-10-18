package com.twitter.botmaker.compiler;

import java.util.Collection;
import java.util.Map;

import com.google.common.base.Optional;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableSet;

public interface DerivedFeatureProvider {

  Optional<DerivedFeature> get(String derivedFeatureName);

  Collection<DerivedFeature> all();

  boolean containsFingerprint(Fingerprint fingerprint);

  static final DerivedFeatureProvider EMPTY = new DerivedFeatureProvider() {
    @Override
    public Optional<DerivedFeature> get(String derivedFeatureName) {
      return Optional.absent();
    }

    @Override
    public Collection<DerivedFeature> all() {
      return ImmutableList.of();
    }

    @Override
    public boolean containsFingerprint(Fingerprint fingerprint) {
      return fingerprint == Fingerprint.EMPTY;
    }
  };

  static DerivedFeatureProvider of(Map<String, DerivedFeature> derivedFeatures) {
    if (derivedFeatures.size() == 0) {
      return EMPTY;
    }

    ImmutableSet.Builder<Fingerprint> builder = ImmutableSet.builder();
    for (DerivedFeature derivedFeature : derivedFeatures.values()) {
      builder.add(derivedFeature.getFingerprint());
    }

    ImmutableSet<Fingerprint> fingerprints = builder.build();

    return new DerivedFeatureProvider() {
      @Override
      public Optional<DerivedFeature> get(String derivedFeatureName) {
        return Optional.fromNullable(derivedFeatures.get(derivedFeatureName));
      }

      @Override
      public Collection<DerivedFeature> all() {
        return derivedFeatures.values();
      }

      @Override
      public boolean containsFingerprint(Fingerprint fingerprint) {
        return fingerprints.contains(fingerprint);
      }
    };
  }

  static DerivedFeatureProvider of(DerivedFeature... derivedFeatures) {
    if (derivedFeatures.length == 0) {
      return EMPTY;
    }

    ImmutableMap.Builder<String, DerivedFeature> builder = ImmutableMap.builder();
    for (DerivedFeature derivedFeature : derivedFeatures) {
      builder.put(derivedFeature.getName(), derivedFeature);
    }

    return of(builder.build());
  }

  static DerivedFeatureProvider of(Collection<DerivedFeature> derivedFeatures) {
    if (derivedFeatures.size() == 0) {
      return EMPTY;
    }

    ImmutableMap.Builder<String, DerivedFeature> builder = ImmutableMap.builder();
    for (DerivedFeature derivedFeature : derivedFeatures) {
      builder.put(derivedFeature.getName(), derivedFeature);
    }

    return of(builder.build());
  }
}
