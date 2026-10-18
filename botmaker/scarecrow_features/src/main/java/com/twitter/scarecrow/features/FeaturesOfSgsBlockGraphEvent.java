package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.socialgraph.thriftjava.Action;
import com.twitter.socialgraph.thriftjava.BlockGraphEvent;
import com.twitter.socialgraph.thriftjava.BlockType;
import com.twitter.socialgraph.thriftjava.LogEventContext;
import com.twitter.socialgraph.thriftjava.SrcTargetRequest;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfSgsBlockGraphEvent extends FeatureMapExtractor {

  private final BlockGraphEvent event;
  private final LogEventContext context;
  private final Action action;
  private final FeaturesOfTwitterContext twitterContexts;

  public FeaturesOfSgsBlockGraphEvent(BlockGraphEvent event,
    LogEventContext context,
    Action action,
    FeaturesOfTwitterContext twitterContexts
  ) {
    this.event = Preconditions.checkNotNull(event);
    this.context =  Preconditions.checkNotNull(context);
    this.action = Preconditions.checkNotNull(action);
    this.twitterContexts =  Preconditions.checkNotNull(twitterContexts);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    FeaturesOfSgsWriteRequestResult.addFeaturesFromContext(builder, context);

    SrcTargetRequest request = event.getResult().getRequest();

    builder.putValue(REQUIRED, BotMakerFeatures.sourceId, request.getSource());
    builder.putValue(REQUIRED, BotMakerFeatures.targetId, request.getTarget());

    if (action == Action.BLOCK) {
      builder.putValue(REQUIRED, BotMakerFeatures.victimId, request.getSource());
      builder.putValue(REQUIRED, BotMakerFeatures.spammerId, request.getTarget());
    }

    builder.putValue(OPTIONAL, BotMakerFeatures.targetFollowingSource,
        event.isWas_target_following_src());
    builder.putValue(OPTIONAL, BotMakerFeatures.sourceFollowingTarget,
        event.isWas_src_following_target());

    if (event.isSetBlock_context()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.blockType,
          event.getBlock_context().getBlock_type().toString());
    } else {
      builder.putValue(OPTIONAL, BotMakerFeatures.blockType,
          BlockType.USER_REQUESTED.toString());
    }

    twitterContexts.applyAll(builder);
  }
}
