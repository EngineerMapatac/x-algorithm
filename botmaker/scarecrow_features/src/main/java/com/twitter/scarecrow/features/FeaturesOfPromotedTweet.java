package com.twitter.scarecrow.features;

import org.apache.thrift.TDeserializer;
import org.apache.thrift.TException;
import org.apache.thrift.protocol.TCompactProtocol;

import com.twitter.ads.db.write.WriteOperation;
import com.twitter.ads.entities.db.ChangeNotification;
import com.twitter.ads.entities.db.PromotedTweet;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;
import static com.twitter.botmaker.FeatureModifier.OPTIONAL;

public class FeaturesOfPromotedTweet extends FeatureMapExtractor {
  private static final ThreadLocal<TDeserializer> COMPACT_THRIFT_DESERIALIZER =
      ThreadLocal.withInitial(() -> new TDeserializer(new TCompactProtocol.Factory()));

  private final ChangeNotification notif;

  public FeaturesOfPromotedTweet(ChangeNotification notif) {
    this.notif = notif;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    final long timestampMs;
    if (notif.isSetCreated_at()) {
      timestampMs = notif.getCreated_at().getPosixTime() * 1000L;
    } else {
      timestampMs = 0;
    }
    builder.putValue(REQUIRED, BotMakerFeatures.timestampMs, timestampMs);

    WriteOperation operationType = notif.getOperation_type();
    if (operationType != null) {
      builder.putValue(OPTIONAL, BotMakerFeatures.adWriteOperationType, operationType.name());
    }

    PromotedTweet promotedTweet = promotedTweet(notif);
    if (promotedTweet.isSetTweet_id()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.tweetId, promotedTweet.getTweet_id());
    }
    if (promotedTweet.isSetAccount_id()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.adAccountId, promotedTweet.getAccount_id());
    }
    if (promotedTweet.isSetLine_item_id()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.adLineItemId, promotedTweet.getLine_item_id());
    }
    if (promotedTweet.isSetUser_id()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.actorId, promotedTweet.getUser_id());
      builder.putValue(OPTIONAL, BotMakerFeatures.spammerId, promotedTweet.getUser_id());
    }
    if (promotedTweet.isSetApproval_status()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.adApprovalStatus,
          promotedTweet.getApproval_status().name());
    }
    if (promotedTweet.isSetDeleted()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.adDeleted, promotedTweet.isDeleted());
    }
    if (promotedTweet.isSetPromotion_source()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.adPromotionSource,
          promotedTweet.getPromotion_source().name());
    }
  }

  private static PromotedTweet promotedTweet(ChangeNotification notif) throws TException {
    PromotedTweet promotedTweet = new PromotedTweet();
    COMPACT_THRIFT_DESERIALIZER.get().deserialize(promotedTweet, notif.getEntity_object());
    return promotedTweet;
  }
}
