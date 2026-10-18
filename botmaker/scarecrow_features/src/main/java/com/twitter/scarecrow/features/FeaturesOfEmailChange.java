package com.twitter.scarecrow.features;

import com.twitter.emailstorage.api.thriftjava.EmailChange;
import com.twitter.emailstorage.api.thriftjava.Operation;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfEmailChange extends FeatureMapExtractor {
  private final EmailChange emailChange;
  private final FeaturesOfTwitterContext twitterContextFeatures;

  public FeaturesOfEmailChange(
      EmailChange emailChange, FeaturesOfTwitterContext twitterContextFeatures) {
    this.emailChange = emailChange;
    this.twitterContextFeatures = twitterContextFeatures;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(REQUIRED, BotMakerFeatures.actorId, emailChange.getUserId())
        .putValue(REQUIRED, BotMakerFeatures.spammerId, emailChange.getUserId())
        .putValue(REQUIRED, BotMakerFeatures.forUserId, emailChange.getUserId())
        .putValue(REQUIRED, BotMakerFeatures.eventTimestampMs, emailChange.getTimestampMs())
        .putValue(REQUIRED, BotMakerFeatures.operation, emailChange.getOperation().name());
    if (emailChange.getOperation() == Operation.DELETE) {
      builder.putValue(OPTIONAL, BotMakerFeatures.encryptedEmailAddressDeleted,
          emailChange.getEncryptedEmailAddress());
    } else {
      builder.putValue(OPTIONAL, BotMakerFeatures.encryptedEmailAddressNow,
          emailChange.getEncryptedEmailAddress());
    }
    twitterContextFeatures.applyAll(builder);
  }
}
