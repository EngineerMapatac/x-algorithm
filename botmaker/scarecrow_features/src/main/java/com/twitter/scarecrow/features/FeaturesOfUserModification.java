package com.twitter.scarecrow.features;

import com.google.common.base.Preconditions;

import com.twitter.gizmoduck.thriftjava.UpdateDiffItem;
import com.twitter.gizmoduck.thriftjava.UserAuditData;
import com.twitter.gizmoduck.thriftjava.UserModification;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfUserModification extends FeatureMapExtractor {

  private final UserModification userModification;
  private final FeaturesOfTwitterContext twitterContextFeatures;

  public FeaturesOfUserModification(
      UserModification userModification,
      FeaturesOfTwitterContext twitterContextFeatures
  ) {
    this.userModification = Preconditions.checkNotNull(userModification);
    this.twitterContextFeatures = Preconditions.checkNotNull(twitterContextFeatures);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {

    builder.putValue(REQUIRED, BotMakerFeatures.actorId, userModification.getUser_id())
        .putValue(REQUIRED, BotMakerFeatures.spammerId, userModification.getUser_id())
        .putValue(REQUIRED, BotMakerFeatures.forUserId, userModification.getFor_user_id())
        .putValue(REQUIRED, BotMakerFeatures.eventTimestampMs,
            userModification.getUpdated_at_msec());
    if (userModification.isSetUser_type()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.userType, userModification.getUser_type().name());
    }
    if (userModification.isSetDestroy()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.userDestroy, userModification.getDestroy());
    }
    if (userModification.isSetUpdate()) {
      for (UpdateDiffItem updateDiffItem : userModification.getUpdate()) {
        if (updateDiffItem.isSetField_name()) {
          String fieldName = updateDiffItem.getField_name();
          if (updateDiffItem.isSetBefore()) {
            builder.putValue(OPTIONAL, fieldName + ".before", updateDiffItem.getBefore());
          }
          if (updateDiffItem.isSetAfter()) {
            builder.putValue(OPTIONAL, fieldName + ".after", updateDiffItem.getAfter());
          }
        }
      }
    }
    if (userModification.isSetUser_audit_data()) {
      UserAuditData auditData = userModification.getUser_audit_data();
      if (auditData.isSetKnown_device_token()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.knownDeviceToken,
            auditData.getKnown_device_token());
      }
      if (auditData.isSetSession_hash()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.sessionHash, auditData.getSession_hash());
      }
      if (auditData.isSetActor_name()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.actorName, auditData.getActor_name());
      }
    }
    if (userModification.isSetClient_id()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.clientId, userModification.getClient_id());
    }

    twitterContextFeatures.applyAll(builder);
  }
}
