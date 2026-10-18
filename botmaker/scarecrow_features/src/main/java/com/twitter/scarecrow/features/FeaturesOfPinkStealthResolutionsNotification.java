package com.twitter.scarecrow.features;

import java.util.List;
import java.util.Map;

import com.google.common.collect.Iterables;
import com.google.common.collect.Lists;
import com.google.common.collect.Maps;

import com.twitter.pink_floyd.thrift.ConstructionRequestContext;
import com.twitter.pink_floyd.thrift.PinkStealthResolutionsNotification;
import com.twitter.pink_floyd.thrift.Resolution;
import com.twitter.pink_floyd.thrift.StealthResolution;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfPinkStealthResolutionsNotification extends FeatureMapExtractor {
  private final PinkStealthResolutionsNotification notification;

  public FeaturesOfPinkStealthResolutionsNotification(
      PinkStealthResolutionsNotification notification
  ) {
    this.notification = notification;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(REQUIRED, BotMakerFeatures.firstHopUrl, notification.getUrl());
    builder.putValue(REQUIRED, BotMakerFeatures.timestampMs, notification.getTimestampMs());

    final List<StealthResolution> stealthResolutions =
        notification.getResolutions().getStealthResolutions();
    final StealthResolutionsInfo stealthResolutionsInfo =
        new StealthResolutionsInfo(stealthResolutions);

    builder.putCollection(
        REQUIRED,
        BotMakerFeatures.statusCodes,
        stealthResolutionsInfo.getStatusCodes(),
        String.class);
    builder.putValue(
        REQUIRED,
        BotMakerFeatures.redirectionChains,
        stealthResolutionsInfo.getRedirectionChains());
    builder.putValue(
        REQUIRED,
        BotMakerFeatures.redirectionChainIps,
        stealthResolutionsInfo.getRedirectionChainIps());
    builder.putValue(
        REQUIRED,
        BotMakerFeatures.originalIps,
        stealthResolutionsInfo.getOriginalIps());
    builder.putValue(
        REQUIRED,
        BotMakerFeatures.expirationsMs,
        stealthResolutionsInfo.getExpirationsMs());
    builder.putValue(
        REQUIRED,
        BotMakerFeatures.userAgents,
        stealthResolutionsInfo.getUserAgents());
    builder.putValue(
        REQUIRED,
        BotMakerFeatures.tags,
        stealthResolutionsInfo.getTags());

    if (notification.isSetContext()) {
      final ConstructionRequestContext context = notification.getContext();
      if (context.isSetTcoCreation()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.isTcoCreation, context.isTcoCreation());
      }
      if (context.isSetTcoPrivacyStatus()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.tcoPrivacyStatus,
            context.getTcoPrivacyStatus().toString());
      }
      if (context.isSetTcoUserId()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.tcoUserId, context.getTcoUserId());
      }
      if (context.isSetTcoTweetId()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.tweetId, context.getTcoTweetId());
      }
      if (context.isSetTcoDmId()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.dmId, context.getTcoDmId());
      }
    }
  }

  static class StealthResolutionsInfo {
    private final List<String> statusCodes;
    List<String> getStatusCodes() {
      return statusCodes;
    }
    private final Map<String, List<String>> redirectionChains;
    Map<String, List<String>> getRedirectionChains() {
      return redirectionChains;
    }
    private final Map<String, List<String>> redirectionChainIps;
    Map<String, List<String>> getRedirectionChainIps() {
      return redirectionChainIps;
    }
    private final Map<String, List<String>> originalIps;
    Map<String, List<String>> getOriginalIps() {
      return originalIps;
    }
    private final Map<String, Long> expirationsMs;
    Map<String, Long> getExpirationsMs() {
      return expirationsMs;
    }
    private final Map<String, String> userAgents;
    Map<String, String> getUserAgents() {
      return userAgents;
    }
    private final Map<String, String> tags;
    Map<String, String> getTags() {
      return tags;
    }

    StealthResolutionsInfo(List<StealthResolution> stealthResolutions) {
      statusCodes = Lists.newArrayList();
      redirectionChains = Maps.newHashMap();
      redirectionChainIps = Maps.newHashMap();
      originalIps = Maps.newHashMap();
      expirationsMs = Maps.newHashMap();
      userAgents = Maps.newHashMap();
      tags = Maps.newHashMap();
      for (int index = 0; index < stealthResolutions.size(); ++index) {
        final String indexStr = Integer.toString(index);
        StealthResolution stealthResolution = stealthResolutions.get(index);
        final Resolution resolution = stealthResolution.getResolution();
        statusCodes.add(resolution.getStatus().toString());
        if (resolution.isSetRedirectionChain()) {
          redirectionChains.put(indexStr, resolution.getRedirectionChain());
        }
        if (resolution.isSetRedirectionChainIps()) {
          redirectionChainIps.put(
              indexStr,
              Lists.newLinkedList(Iterables.concat(resolution.getRedirectionChainIps())));
        }
        if (resolution.isSetOriginalIps()) {
          originalIps.put(indexStr, resolution.getOriginalIps());
        }
        if (resolution.isSetExpiresAt()) {
          expirationsMs.put(indexStr, resolution.getExpiresAt());
        }
        if (stealthResolution.isSetUserAgent()) {
          userAgents.put(indexStr, stealthResolution.getUserAgent());
        }
        if (stealthResolution.isSetTag()) {
          tags.put(indexStr, stealthResolution.getTag().toString());
        }
      }
    }
  }
}
