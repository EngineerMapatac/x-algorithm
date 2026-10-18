package com.twitter.scarecrow.features;

import java.util.List;
import java.util.stream.Collectors;

import com.google.common.base.Preconditions;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.EnumExtractors;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.spam.rtf.thriftjava.ModelMetadataV1;
import com.twitter.spam.rtf.thriftjava.SafetyLabel;
import com.twitter.spam.rtf.thriftjava.ToolAction;
import com.twitter.spam.rtf.thriftjava.TweetSafetyLabelEvent;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;
import static com.twitter.spam.rtf.thriftjava.SafetyLabelSource._Fields.BOT_MAKER_ACTION;
import static com.twitter.spam.rtf.thriftjava.SafetyLabelSource._Fields.TOOL_ACTION;

public class FeaturesOfTweetSafetyLabelEvent extends FeatureMapExtractor {
  private final TweetSafetyLabelEvent event;

  public FeaturesOfTweetSafetyLabelEvent(TweetSafetyLabelEvent event) {
    this.event = Preconditions.checkNotNull(event);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder
        .putValue(REQUIRED, BotMakerFeatures.tweetId, event.getTweetId())
        .putExtractor(REQUIRED, BotMakerFeatures.action,
            new EnumExtractors.Name<>(event.getActionType()))
        .putExtractor(REQUIRED, BotMakerFeatures.safetyLabel,
            new EnumExtractors.Name<>(event.getLabelType()))
        .putValue(OPTIONAL, BotMakerFeatures.eventTimestampMs, event.getTimestampMs());

    if (event.isSetLabel()) {
      SafetyLabel label = event.getLabel();

      builder
          .putValue(OPTIONAL, BotMakerFeatures.source, label.getSource())
          .putValue(OPTIONAL, BotMakerFeatures.score, label.isSetScore() ? label.getScore() : null)
          .putValue(OPTIONAL, BotMakerFeatures.enableHoldbackExperiment,
              label.isSetHoldback_experiment() ? label.isHoldback_experiment() : null);

      if (label.isSetApplicable_users()) {
        List<Long> applicableUserIds = label.getApplicable_users().stream().map(
            user -> user.user_id).collect(Collectors.toList());
        builder.putValue(OPTIONAL, BotMakerFeatures.applicableUsers, applicableUserIds);
      }

      if (label.isSetModel_metadata() && label.getModel_metadata().getModelMetadataV1() != null) {
        ModelMetadataV1 metadata = label.getModel_metadata().getModelMetadataV1();

        builder
            .putValue(OPTIONAL, BotMakerFeatures.modelVersion,
                metadata.isSetVersion() ? metadata.getVersion() : null)
            .putValue(
                OPTIONAL, BotMakerFeatures.calibratedLanguage, metadata.getCalibratedLanguage());
      }

      if (label.isSetSafetyLabelSource()) {
        if (label.getSafetyLabelSource().getSetField() == TOOL_ACTION) {
          ToolAction toolAction = label.getSafetyLabelSource().getToolAction();
          builder
              .putValue(OPTIONAL, BotMakerFeatures.actorLdap, toolAction.getActorLdap())
              .putExtractor(OPTIONAL, BotMakerFeatures.tool,
                  toolAction.isSetAgentTool()
                      ? new EnumExtractors.Name<>(toolAction.getAgentTool()) : null);
        }

        if (label.getSafetyLabelSource().getSetField() == BOT_MAKER_ACTION) {
          builder
              .putValue(OPTIONAL, BotMakerFeatures.botId,
                  label.getSafetyLabelSource().getBotMakerAction().getRuleId());
        }
      }
    }
  }
}
