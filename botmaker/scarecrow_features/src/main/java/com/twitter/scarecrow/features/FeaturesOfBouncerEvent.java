package com.twitter.scarecrow.features;

import java.util.stream.Collectors;

import com.google.common.base.Preconditions;

import com.twitter.bouncer.logging.thriftjava.EnrollmentEvent;
import com.twitter.bouncer.thriftjava.Enrollment;
import com.twitter.bouncer.thriftjava.InternalActor;
import com.twitter.bouncer.thriftjava.Lookup;
import com.twitter.bouncer.thriftjava.Target;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.EnumExtractors;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfBouncerEvent extends FeatureMapExtractor {
  private final EnrollmentEvent event;

  public FeaturesOfBouncerEvent(EnrollmentEvent event) {
    this.event = Preconditions.checkNotNull(event);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    final Lookup lookup = Preconditions.checkNotNull(
      event.getLookup(),
      String.format("missing lookup: %s", event)
    );
    final Enrollment enrollment = Preconditions.checkNotNull(
      event.getEnrollment(),
      String.format("missing enrollment: %s", event)
    );

    processEventMetadata(builder);
    processLookup(builder, lookup);
    processEnrollment(builder, enrollment);
  }

  private void processLookup(
    FeatureMapBuilder builder,
    final Lookup lookup
  ) throws Exception {
    Target target = lookup.getTarget();
    if (target.isSet(Target._Fields.USER)) {
      builder.putValue(
        REQUIRED,
        BotMakerFeatures.userId,
        target.getUser().getUser_id()
      );
    } else if (target.isSet(Target._Fields.FEATURE)) {
      builder.putValue(
        REQUIRED,
        BotMakerFeatures.userId,
        target.getFeature().getUser_id()
      );
      builder.putValue(
        REQUIRED,
        BotMakerFeatures.requestFeature,
        target.getFeature().getFeature()
      );
    }
  }

  private void processEventMetadata(
   FeatureMapBuilder builder
  ) throws Exception {
    builder
      .putValue(
        REQUIRED,
        BotMakerFeatures.eventTimestampMs,
        event.getOccurred_at_msec())
      .putExtractor(
        REQUIRED,
        BotMakerFeatures.updateType,
        EnumExtractors.name(event.getEvent()));
  }

  private void processEnrollment(
    FeatureMapBuilder builder,
    final Enrollment enrollment
  ) throws Exception {
    builder
      .putValue(
        REQUIRED,
        BotMakerFeatures.template,
        enrollment.getTemplate().getId().getId())
      .putValue(
        REQUIRED,
        BotMakerFeatures.policyInViolation,
        enrollment.getReferring_tags() == null ? null : enrollment.getReferring_tags().stream()
            .map(Object::toString).collect(Collectors.toSet())
      );

    if (enrollment.isSetReleased_at_msec()) {
      builder.putValue(
        OPTIONAL,
        BotMakerFeatures.releasedAtMs,
        enrollment.getReleased_at_msec()
      );
    }

    if (enrollment.isSetActor() && enrollment.getActor().isSet(InternalActor._Fields.RULE)) {
      builder.putValue(
          OPTIONAL,
          BotMakerFeatures.botId,
          enrollment.getActor().getRule().getRule_id()
      );
    }
  }

}
