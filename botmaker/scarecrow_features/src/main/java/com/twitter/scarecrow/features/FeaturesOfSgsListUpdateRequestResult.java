package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.socialgraph.thriftjava.ListIdentifier;
import com.twitter.socialgraph.thriftjava.ListUpdateRequest;
import com.twitter.socialgraph.thriftjava.ListUpdateRequestResult;
import com.twitter.socialgraph.thriftjava.ListUpdateResult;
import com.twitter.socialgraph.thriftjava.LogEventContext;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfSgsListUpdateRequestResult extends FeatureMapExtractor {

  private final ListUpdateRequest request;
  private final ListUpdateResult result;
  private final LogEventContext context;
  private final FeaturesOfTwitterContext twitterContexts;

  public FeaturesOfSgsListUpdateRequestResult(
      ListUpdateRequestResult requestResult,
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

    if (request.getId().getSetField() == ListIdentifier._Fields.LIST_ID) {
      builder.putValue(REQUIRED, BotMakerFeatures.listId, request.getId().getList_id());
    } else {
      builder.putValue(REQUIRED, BotMakerFeatures.listId, result.getResult().getList_id());
      builder.putValue(REQUIRED,
          BotMakerFeatures.ownerId, request.getId().getOwner_slug_pair().getUser_id());
      builder.putValue(REQUIRED,
          BotMakerFeatures.listSlug, request.getId().getOwner_slug_pair().getList_slug());
    }

    builder.putValue(OPTIONAL, BotMakerFeatures.newName, request.getNew_name());
    builder.putValue(OPTIONAL, BotMakerFeatures.newDescription, request.getNew_description());
    builder.putValue(OPTIONAL, BotMakerFeatures.isPrivate, request.isIs_private());

    builder.putValue(REQUIRED, BotMakerFeatures.listSlug, result.getResult().getList_slug());

    twitterContexts.applyAll(builder);
  }
}
