package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.socialgraph.thriftjava.Action;
import com.twitter.socialgraph.thriftjava.FollowGraphEvent;
import com.twitter.socialgraph.thriftjava.FollowType;
import com.twitter.socialgraph.thriftjava.LogEventContext;
import com.twitter.socialgraph.thriftjava.SrcTargetRequest;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfSgsFollowGraphEvent extends FeatureMapExtractor {

  private final FollowGraphEvent event;
  private final LogEventContext context;
  private final Action action;
  private final FeaturesOfTwitterContext twitterContexts;

  public FeaturesOfSgsFollowGraphEvent(
      FollowGraphEvent event,
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

    if (action == Action.FOLLOW) {
      builder.putValue(REQUIRED, BotMakerFeatures.spammerId, request.getSource());
      builder.putValue(REQUIRED, BotMakerFeatures.victimId, request.getTarget());
      if (event.isSetIs_bulk_follow()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.isBulkFollow, event.isIs_bulk_follow());
      }
      if (event.isSetIs_nux_follow()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.isNuxFollow, event.isIs_nux_follow());
      }
    }
    if (event.isSetAccepted_follow_request()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.isAcceptedFollowRequest,
          event.isAccepted_follow_request());
    }
    if (event.isSetFollow_type()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.isCreateFollowRequest,
          event.getFollow_type().equals(FollowType.CREATE_FOLLOW_REQUEST));
    }

    if (event.isSetDoes_target_follow_source()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.mutualFollow,
          event.isDoes_target_follow_source());
    }
    twitterContexts.applyAll(builder);
  }
}
