
package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.socialgraph.thriftjava.Action;
import com.twitter.socialgraph.thriftjava.LogEventContext;
import com.twitter.socialgraph.thriftjava.MuteGraphEvent;
import com.twitter.socialgraph.thriftjava.MuteType;
import com.twitter.socialgraph.thriftjava.SrcTargetRequest;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfSgsMuteGraphEvent extends FeatureMapExtractor {

  private final MuteGraphEvent event;
  private final LogEventContext context;
  private final Action action;
  private final FeaturesOfTwitterContext twitterContexts;

  public FeaturesOfSgsMuteGraphEvent(
      MuteGraphEvent event,
      LogEventContext context,
      Action action,
      FeaturesOfTwitterContext twitterContexts
  ) {
    this.event = Preconditions.checkNotNull(event);
    this.context =  Preconditions.checkNotNull(context);
    this.action = Preconditions.checkNotNull(action);
    this.twitterContexts = Preconditions.checkNotNull(twitterContexts);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    FeaturesOfSgsWriteRequestResult.addFeaturesFromContext(builder, context);

    SrcTargetRequest request = event.getResult().getRequest();

    builder.putValue(REQUIRED, BotMakerFeatures.sourceId, request.getSource());
    builder.putValue(REQUIRED, BotMakerFeatures.targetId, request.getTarget());

    if (action == Action.MUTE) {
      builder.putValue(REQUIRED, BotMakerFeatures.victimId, request.getSource());
      builder.putValue(REQUIRED, BotMakerFeatures.spammerId, request.getTarget());
    } else {
      builder.putValue(OPTIONAL, BotMakerFeatures.wasTargetSmartMuted,
          event.isWas_target_smart_muted());
    }

    if (event.isSetMute_context()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.muteType,
          event.getMute_context().getMute_type().toString());
    } else if (action == Action.MUTE) {
      builder.putValue(OPTIONAL, BotMakerFeatures.muteType,
          MuteType.MANUAL.toString());
    }

    twitterContexts.applyAll(builder);
  }
}
