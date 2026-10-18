package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.socialgraph.thriftjava.LogEventContext;
import com.twitter.socialgraph.thriftjava.SrcTargetRequest;
import com.twitter.socialgraph.thriftjava.WriteRequestResult;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureExtractionException;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfSgsWriteRequestResult extends FeatureMapExtractor {

  private final WriteRequestResult result;
  private final LogEventContext context;
  private final FeaturesOfTwitterContext twitterContexts;

  static void addFeaturesFromContext(FeatureMapBuilder builder, LogEventContext logContext)
      throws FeatureExtractionException {
    builder.putValue(REQUIRED, BotMakerFeatures.timestampMs, logContext.getTimestamp());
    builder.putValue(OPTIONAL, BotMakerFeatures.applicationId,
        logContext.getClient_application_id());
    builder.putValue(OPTIONAL, BotMakerFeatures.auditIp, logContext.getClient_ip());
    builder.putValue(OPTIONAL, BotMakerFeatures.authenticatedUserId,
        logContext.getAuthenticated_user_id());
    builder.putValue(OPTIONAL, BotMakerFeatures.actorId, logContext.getLogged_in_user_id());
  }

  public FeaturesOfSgsWriteRequestResult(
      WriteRequestResult result,
      LogEventContext context,
      FeaturesOfTwitterContext twitterContexts
  ) {
    this.result = Preconditions.checkNotNull(result);
    this.context =  Preconditions.checkNotNull(context);
    this.twitterContexts =  Preconditions.checkNotNull(twitterContexts);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    addFeaturesFromContext(builder, context);

    SrcTargetRequest request = result.getRequest();
    builder.putValue(REQUIRED, BotMakerFeatures.sourceId, request.getSource());
    builder.putValue(REQUIRED, BotMakerFeatures.targetId, request.getTarget());

    twitterContexts.applyAll(builder);
  }
}
