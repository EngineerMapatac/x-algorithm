package com.twitter.scarecrow.features;

import com.twitter.gizmoduck.thriftjava.Account;
import com.twitter.gizmoduck.thriftjava.Profile;
import com.twitter.gizmoduck.thriftjava.Safety;
import com.twitter.gizmoduck.thriftjava.User;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfUserCreate extends FeatureMapExtractor {

  private final User user;
  private final FeaturesOfTwitterContext twitterContextFeatures;

  public FeaturesOfUserCreate(User user, FeaturesOfTwitterContext twitterContextFeatures) {
    this.user = user;
    this.twitterContextFeatures = twitterContextFeatures;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    final long userId = user.getId();
    builder.putValue(REQUIRED, BotMakerFeatures.actorId, userId)
        .putValue(REQUIRED, BotMakerFeatures.spammerId, userId)
        .putPositive(OPTIONAL, BotMakerFeatures.eventTimestampMs, user.getCreated_at_msec())
        .putValue(REQUIRED, BotMakerFeatures.userType, user.getUser_type().name());

    if (user.isSetAccount()) {
      final Account account = user.getAccount();
      builder.putValue(OPTIONAL, BotMakerFeatures.creationIp, account.getCreation_ip())
          .putValue(OPTIONAL, BotMakerFeatures.accountLanguage, account.getLanguage());
      if (account.isSetCreated_via()) {
        builder.putValue(REQUIRED, BotMakerFeatures.accountCreatedVia, account.getCreated_via())
            .putPositive(OPTIONAL, BotMakerFeatures.applicationId,
                BotMakerFeatures.transformCreatedViaToApplicationId(account.getCreated_via()));
      }
    }
    if (user.isSetProfile()) {
      final Profile profile = user.getProfile();
      builder.putValue(OPTIONAL, BotMakerFeatures.firstAndLastName, profile.getName());
      builder.putValue(OPTIONAL, BotMakerFeatures.screenName, profile.getScreen_name());
    }
    if (user.isSetSafety()) {
      final Safety safety = user.getSafety();
      if (safety.isSetSignup_creation_source()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.signupCreationSource,
            safety.signup_creation_source.name());
      }
    }

    twitterContextFeatures.applyAll(builder);
  }
}
