package com.twitter.botmaker.compiler;

import java.util.Objects;
import java.util.Optional;

public final class UsageUnit {

  private final Optional<Fingerprint> derivedFeatureFingerprint;
  private final Optional<Fingerprint> compilerUnitFingerprint;
  private final Optional<String> inputFeatureName;
  private final int hashCode;

  private UsageUnit(
      Optional<Fingerprint> derivedFeatureFingerprint,
      Optional<Fingerprint> compilerUnitFingerprint,
      Optional<String> inputFeatureName
  ) {
    this.derivedFeatureFingerprint = derivedFeatureFingerprint;
    this.compilerUnitFingerprint = compilerUnitFingerprint;
    this.inputFeatureName = inputFeatureName;
    this.hashCode = Objects.hash(
        derivedFeatureFingerprint,
        compilerUnitFingerprint,
        inputFeatureName
    );
  }

  public static UsageUnit ofDerivedFeatureFingerprint(
      Fingerprint derivedFeatureFingerprint
  ) {
    Objects.requireNonNull(derivedFeatureFingerprint);
    return new UsageUnit(
        Optional.of(derivedFeatureFingerprint),
        Optional.empty(),
        Optional.empty());
  }

  public static UsageUnit ofCompilerUnitFingerprint(
      Fingerprint compilerUnitFingerprint
  ) {
    Objects.requireNonNull(compilerUnitFingerprint);
    return new UsageUnit(
      Optional.empty(),
      Optional.of(compilerUnitFingerprint),
      Optional.empty()
    );
  }

  public static UsageUnit ofInputFeatureName(
      String inputFeatureName
  ) {
    Objects.requireNonNull(inputFeatureName);
    return new UsageUnit(
        Optional.empty(),
        Optional.empty(),
        Optional.of(inputFeatureName)
    );
  }

  public boolean hasDerivedFeature() {
    return derivedFeatureFingerprint.isPresent();
  }

  public boolean hasCompilerUnit() {
    return compilerUnitFingerprint.isPresent();
  }

  public boolean hasInputFeature() {
    return inputFeatureName.isPresent();
  }

  public Optional<Fingerprint> getDerivedFeatureFingerprint() {
    return derivedFeatureFingerprint;
  }

  public Optional<Fingerprint> getCompilerUnitFingerprint() {
    return compilerUnitFingerprint;
  }

  public Optional<String> getInputFeatureName() {
    return inputFeatureName;
  }

  @Override
  public boolean equals(Object object) {
    if (this == object) {
      return true;
    } else if (!(object instanceof UsageUnit)) {
      return false;
    } else {
      UsageUnit that = (UsageUnit) object;
      return Objects.equals(this.derivedFeatureFingerprint, that.derivedFeatureFingerprint)
          && Objects.equals(this.compilerUnitFingerprint, that.compilerUnitFingerprint)
          && Objects.equals(this.inputFeatureName, that.inputFeatureName);
    }
  }

  @Override
  public int hashCode() {
    return hashCode;
  }

}
