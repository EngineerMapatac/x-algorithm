package com.twitter.scarecrow.features;

import java.util.List;
import java.util.stream.Collectors;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.AudioSpaceTopic;
import com.twitter.ubs.thriftjava.TopicsUpdatedEvent;

public class FeaturesOfAudioSpaceTopicsUpdatedEvent extends FeatureMapExtractor {

  private final TopicsUpdatedEvent topicsUpdatedEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfAudioSpaceTopicsUpdatedEvent(
      TopicsUpdatedEvent topicsUpdatedEvent,
      AudioSpaceBaseEvent baseEvent
  ) {
    this.topicsUpdatedEvent = topicsUpdatedEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (topicsUpdatedEvent.isSetTopics()) {
      List<String> topicIds = topicsUpdatedEvent.getTopics().stream()
          .map(AudioSpaceTopic::getTopic_id).collect(Collectors.toList());
      builder.putCollection(FeatureModifier.OPTIONAL, BotMakerFeatures.topics, topicIds,
          String.class);
    }
  }
}
