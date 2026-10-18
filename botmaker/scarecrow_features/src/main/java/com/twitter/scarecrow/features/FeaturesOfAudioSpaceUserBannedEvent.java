package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.EventActor;
import com.twitter.ubs.thriftjava.PeriscopeUser;
import com.twitter.ubs.thriftjava.UserBannedEvent;

public class FeaturesOfAudioSpaceUserBannedEvent extends FeatureMapExtractor {

  private final UserBannedEvent userBannedEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfAudioSpaceUserBannedEvent(UserBannedEvent userBannedEvent,
                                             AudioSpaceBaseEvent baseEvent) {
    this.userBannedEvent = userBannedEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (userBannedEvent.isSetBanned_user()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userId,
          userBannedEvent.getBanned_user().getUser_id());
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userPeriscopeId,
          userBannedEvent.getBanned_user().getUser_periscope_id());
    }
    if (userBannedEvent.isSetBan_actor()) {
      EventActor banActor = userBannedEvent.getBan_actor();
      if (banActor.isSet(EventActor._Fields.AGENT)) {
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.actorLdap,
            banActor.getAgent().ldap_name);
      } else if (banActor.isSet(EventActor._Fields.THEMSELVES)) {
        PeriscopeUser banActorUser = banActor.getThemselves();
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.actorUserId,
            banActorUser.getUser_id());
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.actorUserPeriscopeId,
            banActorUser.getUser_periscope_id());
      } else if (banActor.isSet(EventActor._Fields.AUTO)) {
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.actorName,
            banActor.getAuto().actor_name);
      }
    }
    if (userBannedEvent.isSetPolicy_in_violation()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.policyInViolation,
          userBannedEvent.getPolicy_in_violation().name());
    }
  }
}
