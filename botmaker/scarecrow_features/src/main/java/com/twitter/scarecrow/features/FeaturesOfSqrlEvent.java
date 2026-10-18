package com.twitter.scarecrow.features;

import org.apache.thrift.TSerializer;

import com.twitter.safety_systems.smyte_bridge.event.thriftjava.Event;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfSqrlEvent extends FeatureMapExtractor {
  private static final ThreadLocal<TSerializer> JSON_SERIALIZER = ThreadLocal.withInitial(() ->
      new TSerializer(new SqrlEventThriftSimpleJSONProtocol.Factory()));

  private final Event sqrlEvent;

  public FeaturesOfSqrlEvent(Event sqrlEvent) {
    this.sqrlEvent = sqrlEvent;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    String json = JSON_SERIALIZER.get().toString(sqrlEvent, "UTF-8");
    builder.putValue(REQUIRED, BotMakerFeatures.name, sqrlEvent.getAction_name());
    builder.putValue(REQUIRED, BotMakerFeatures.timestampMs, sqrlEvent.getTimestamp_ms());
    builder.putValue(REQUIRED, BotMakerFeatures.payload, json);
  }
}
