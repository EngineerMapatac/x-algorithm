package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.socialgraph.thriftjava.ListCreateRequest;
import com.twitter.socialgraph.thriftjava.ListCreateRequestResult;
import com.twitter.socialgraph.thriftjava.ListCreateResult;
import com.twitter.socialgraph.thriftjava.LogEventContext;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfSgsListCreateRequestResult extends FeatureMapExtractor {

  private final ListCreateRequest request;
  private final ListCreateResult result;
  private final LogEventContext context;
  private final FeaturesOfTwitterContext twitterContexts;

  public FeaturesOfSgsListCreateRequestResult(
      ListCreateRequestResult requestResult,
      LogEventContext context,
      FeaturesOfTwitterContext twitterContexts
  ) {
    Preconditions.checkNotNull(requestResult);
    this.request = Preconditions.checkNotNull(requestResult.request);
    this.result = Preconditions.checkNotNull(requestResult.result);
    this.context =  Preconditions.checkNotNull(context);
    this.twitterContexts =  Preconditions.checkNotNull(twitterContexts);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    FeaturesOfSgsWriteRequestResult.addFeaturesFromContext(builder, context);

    builder.putValue(REQUIRED, BotMakerFeatures.name, request.getName());
    builder.putValue(OPTIONAL, BotMakerFeatures.description, request.getDescription());
    builder.putValue(REQUIRED, BotMakerFeatures.isPrivate, request.isIs_private());

    builder.putValue(REQUIRED, BotMakerFeatures.listId, result.getResult().getList_id());
    builder.putValue(REQUIRED, BotMakerFeatures.listSlug, result.getResult().getList_slug());

    twitterContexts.applyAll(builder);
  }
}
