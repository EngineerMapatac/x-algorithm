package com.twitter.scarecrow.features;

import com.twitter.authengine.timeline.thriftjava.AuthTimelineEvent;
import com.twitter.authengine.timeline.thriftjava.AuthTransactionLog;
import com.twitter.authengine.timeline.thriftjava.AuthTransactionMeta;
import com.twitter.authengine.timeline.thriftjava.UrlAudit;
import com.twitter.botmaker.FeatureModifier;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

public class FeaturesOfAuthTransactionLog extends FeatureMapExtractor {
  private final AuthTransactionLog event;

  public FeaturesOfAuthTransactionLog(AuthTransactionLog event) {
    this.event = event;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.token, event.token);
    AuthTransactionMeta meta = event.transaction.meta;
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.timestampMs, meta.timestampMs);
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.endpoint, meta.endpoint);
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.ipAddress, meta.ipAddress);
    if (meta.isSetClientApplicationId()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.applicationId,
          meta.clientApplicationId);
    }
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.knownDeviceToken,
        meta.knownDeviceToken);
    if (meta.isSetUserId()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userId, meta.userId);
    }
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.requestCountryCode,
        meta.countryCode);
    if (meta.isSetHttpResponseCode()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.httpResponseCode,
          meta.httpResponseCode);
    }
    if (meta.isSetRedirect()) {
      UrlAudit urlAudit = meta.redirect;
      if (urlAudit.isSetIsTwitter()) {
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isTwitter, urlAudit.isTwitter);
      }
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.host, urlAudit.host);
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.path, urlAudit.path);
    }
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.httpRequestMethod,
        meta.httpRequestMethod);
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userAgent, meta.userAgent);
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.rootAuthTimelineToken,
        meta.rootAuthTimelineToken);
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.sessionHash, meta.sessionHash);
    builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.ocfServerToken,
        meta.ocfServerToken);

    for (AuthTimelineEvent authTimelineEvent: event.transaction.events) {
      if (authTimelineEvent.isSet(AuthTimelineEvent._Fields.IDENTIFY_ACCOUNT_FAIL)) {
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.rawIdentifier,
            authTimelineEvent.getIdentifyAccountFail().rawIdentifier);
      } else if (authTimelineEvent.isSet(AuthTimelineEvent._Fields.IDENTIFY_ACCOUNT_PASS)) {
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.rawIdentifier,
            authTimelineEvent.getIdentifyAccountPass().rawIdentifier);
      } else if (authTimelineEvent.isSet(AuthTimelineEvent._Fields.IDENTIFY_RESET_ACCOUNT_FAIL)) {
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.rawIdentifier,
            authTimelineEvent.getIdentifyResetAccountFail().rawIdentifier);
      } else if (authTimelineEvent.isSet(AuthTimelineEvent._Fields.IDENTIFY_RESET_ACCOUNT_PASS)) {
        builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.rawIdentifier,
            authTimelineEvent.getIdentifyResetAccountPass().rawIdentifier);
      }
    }
  }
}
