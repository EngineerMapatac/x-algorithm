package com.twitter.scarecrow.features;

import com.twitter.guano.thriftjava.LoginChallengeAction;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.EnumExtractors;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfLoginChallengeAction extends FeatureMapExtractor {

  private final LoginChallengeAction loginChallengeAction;

  public FeaturesOfLoginChallengeAction(LoginChallengeAction loginChallengeAction) {
    this.loginChallengeAction = loginChallengeAction;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder
        .putPositive(
            REQUIRED, BotMakerFeatures.timestampMs,
            1000L * loginChallengeAction.getTimestamp())
        .putPositive(
            REQUIRED, BotMakerFeatures.actorId, loginChallengeAction.getUserId())
        .putPositive(
            OPTIONAL, BotMakerFeatures.byUserId, loginChallengeAction.getByUserId())
        .putValue(
            OPTIONAL, BotMakerFeatures.loginHost, loginChallengeAction.getHost())
        .putValue(
            OPTIONAL, BotMakerFeatures.loginIp, loginChallengeAction.getIp())
        .putValue(
            OPTIONAL, BotMakerFeatures.lastChallengeIdHash,
            loginChallengeAction.getPartialChallengeIdHash())
        .putValue(
            OPTIONAL, BotMakerFeatures.note, loginChallengeAction.getNote())
        .putExtractor(
            OPTIONAL, BotMakerFeatures.action,
            EnumExtractors.name(loginChallengeAction.getAction()))
        .putExtractor(
            OPTIONAL, BotMakerFeatures.lastChallengeType,
            EnumExtractors.name(loginChallengeAction.getChallengeType()))
        .putExtractor(
            OPTIONAL, BotMakerFeatures.lastChallengeCause,
            EnumExtractors.name(loginChallengeAction.getChallengeCause()))
        .putExtractor(
            OPTIONAL, BotMakerFeatures.lockoutStatus,
            EnumExtractors.name(loginChallengeAction.getLockoutStatus()))
        .putPositive(
            OPTIONAL, BotMakerFeatures.applicationId, (long) loginChallengeAction.getClientAppId())
        .putValue(
            OPTIONAL, BotMakerFeatures.rulesTriggered, loginChallengeAction.getRulesTriggered());
  }

}
