package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ubs.thriftjava.AudioSpaceBaseEvent;
import com.twitter.ubs.thriftjava.EventActor;
import com.twitter.ubs.thriftjava.PeriscopeUser;
import com.twitter.ubs.thriftjava.UserUnbannedEvent;

public class FeaturesOfAudioSpaceUserUnbannedEvent extends FeatureMapExtractor {

  private final UserUnbannedEvent userUnbannedEvent;
  private final FeatureMapExtractor baseEventExtractor;

  public FeaturesOfAudioSpaceUserUnbannedEvent(UserUnbannedEvent userUnbannedEvent,
                                               AudioSpaceBaseEvent baseEvent) {
    this.userUnbannedEvent = userUnbannedEvent;
    this.baseEventExtractor = new FeaturesOfAudioSpaceBaseEvent(baseEvent);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    baseEventExtractor.apply(builder);
    if (userUnbannedEvent.isSetUnbanned_user()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userId,
          userUnbannedEvent.getUnbanned_user().getUser_id());
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userPeriscopeId,
          userUnbannedEvent.getUnbanned_user().getUser_periscope_id());
    }
    if (userUnbannedEvent.isSetUnban_actor()) {
      EventActor unbanActor = userUnbannedEvent.getUnban_actor();
      if (unbanActor.isSet(EventActor._Fields.AGENT)) {
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.actorLdap,
            unbanActor.getAgent().ldap_name);
      } else if (unbanActor.isSet(EventActor._Fields.THEMSELVES)) {
        PeriscopeUser periscopeUser = unbanActor.getThemselves();
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.actorUserId,
            periscopeUser.getUser_id());
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.actorUserPeriscopeId,
            periscopeUser.getUser_periscope_id());
      } else if (unbanActor.isSet(EventActor._Fields.AUTO)) {
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.actorName,
            unbanActor.getAuto().actor_name);
      }

    }
  }
}
