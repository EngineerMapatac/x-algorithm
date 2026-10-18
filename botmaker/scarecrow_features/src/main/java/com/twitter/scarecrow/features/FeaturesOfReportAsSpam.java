package com.twitter.scarecrow.features;

import com.twitter.service.gen.sniper.RelationshipUpdate;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureExtractor;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.spam.botmaker_features.UserIdExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfReportAsSpam extends FeatureMapExtractor {

  private final RelationshipUpdate reportAsSpam;

  public FeaturesOfReportAsSpam(RelationshipUpdate reportAsSpam) {
    this.reportAsSpam = reportAsSpam;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    final FeatureExtractor<Long> actorIdExtractor = new UserIdExtractor(
        reportAsSpam.getActor());
    final FeatureExtractor<Long> actedOnIdExtractor = new UserIdExtractor(
        reportAsSpam.getActedOn());
    builder
        .putExtractor(REQUIRED, BotMakerFeatures.actorId, actorIdExtractor)
        .putExtractor(REQUIRED, BotMakerFeatures.victimId, actorIdExtractor)
        .putExtractor(REQUIRED, BotMakerFeatures.spammerId, actedOnIdExtractor)
        .putPositive(OPTIONAL, BotMakerFeatures.applicationId, reportAsSpam.getAppId())
        .putValue(OPTIONAL, BotMakerFeatures.auditIp, reportAsSpam.getIp());
  }

}
