package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.socialgraph.thriftjava.ListAddMemberRequest;
import com.twitter.socialgraph.thriftjava.ListAddMemberRequestResult;
import com.twitter.socialgraph.thriftjava.ListIdentifier;
import com.twitter.socialgraph.thriftjava.LogEventContext;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfSgsListMemberAddRequestResult extends FeatureMapExtractor {

  private final ListAddMemberRequest request;
  private final LogEventContext context;
  private final FeaturesOfTwitterContext twitterContexts;

  public FeaturesOfSgsListMemberAddRequestResult(
      ListAddMemberRequestResult requestResult,
      LogEventContext context,
      FeaturesOfTwitterContext twitterContexts
  ) {
    Preconditions.checkNotNull(requestResult);
    this.request = Preconditions.checkNotNull(requestResult.request);
    this.context =  Preconditions.checkNotNull(context);
    this.twitterContexts =  Preconditions.checkNotNull(twitterContexts);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    FeaturesOfSgsWriteRequestResult.addFeaturesFromContext(builder, context);

    if (request.getId().getSetField() == ListIdentifier._Fields.LIST_ID) {
      builder.putValue(REQUIRED, BotMakerFeatures.listId, request.getId().getList_id());
    } else {
      builder.putValue(REQUIRED,
          BotMakerFeatures.ownerId, request.getId().getOwner_slug_pair().getUser_id());
      builder.putValue(REQUIRED,
          BotMakerFeatures.listSlug, request.getId().getOwner_slug_pair().getList_slug());
    }

    builder.putValue(REQUIRED, BotMakerFeatures.userId, request.getMember_id());

    twitterContexts.applyAll(builder);
  }
}
