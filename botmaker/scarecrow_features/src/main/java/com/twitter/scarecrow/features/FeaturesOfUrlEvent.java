package com.twitter.scarecrow.features;

import com.google.common.base.Optional;
import com.google.common.collect.Iterables;
import com.google.common.collect.Lists;

import com.twitter.pink_floyd.thrift.CardsMetadata;
import com.twitter.pink_floyd.thrift.ConstructError;
import com.twitter.pink_floyd.thrift.Resolution;
import com.twitter.pink_floyd.url_event.thrift.DmInfo;
import com.twitter.pink_floyd.url_event.thrift.DmUrlEvent;
import com.twitter.pink_floyd.url_event.thrift.ExtendedDmAttributes;
import com.twitter.pink_floyd.url_event.thrift.ExtendedTweetAttributes;
import com.twitter.pink_floyd.url_event.thrift.TweetInfo;
import com.twitter.pink_floyd.url_event.thrift.TweetUrlEvent;
import com.twitter.pink_floyd.url_event.thrift.UrlEvent;
import com.twitter.pink_floyd.url_event.thrift.UrlInfo;
import com.twitter.pink_floyd.url_event.thrift.UserInfo;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.EnumExtractors;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfUrlEvent extends FeatureMapExtractor {
  private final UrlEvent req;
  private final Optional<Long> userId;

  public FeaturesOfUrlEvent(UrlEvent req, Optional<Long> userId) {
    this.req = req;
    this.userId = userId;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    if (userId.isPresent()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.tcoUserId,
          userId.get());
      builder.putValue(OPTIONAL, BotMakerFeatures.spammerId,
          userId.get());
    }
    if (req.isSet(UrlEvent._Fields.DM_URL_EVENT)) {
      putDmUrlEventIntoBuilder(req.getDmUrlEvent(), builder);
    } else if (req.isSet(UrlEvent._Fields.TWEET_URL_EVENT)) {
      putTweetUrlEventIntoBuilder(req.getTweetUrlEvent(), builder);
    }
  }

  private void putTweetUrlEventIntoBuilder(
      TweetUrlEvent tweetUrlEvent,
      FeatureMapBuilder builder) throws Exception {
    builder.putValue(REQUIRED, BotMakerFeatures.tcoString,
        tweetUrlEvent.getTco());

    TweetInfo tweetInfo = tweetUrlEvent.getTweetInfo();
    builder.putValue(REQUIRED, BotMakerFeatures.timestampMs,
        tweetInfo.getTweetEventTimeMs());
    builder.putValue(OPTIONAL, BotMakerFeatures.tweetId,
        tweetInfo.getId());

    if (tweetInfo.isSetTweetAttributes()) {
      ExtendedTweetAttributes extendedTweetAttributes = tweetInfo.getTweetAttributes();
      builder.putValue(OPTIONAL, BotMakerFeatures.applicationId,
          extendedTweetAttributes.getClientAppId());
      builder.putValue(OPTIONAL, BotMakerFeatures.isReply,
          extendedTweetAttributes.isIsReply());
      builder.putValue(OPTIONAL, BotMakerFeatures.isRetweet,
          extendedTweetAttributes.isIsRetweet());
      builder.putValue(OPTIONAL, BotMakerFeatures.originalTweetId,
          extendedTweetAttributes.getOriginalTweetId());
    }

    putUrlInfoIntoBuilder(tweetUrlEvent.getUrlInfo(), builder);

    UserInfo userInfo = tweetUrlEvent.getUserInfo();
    builder.putValue(OPTIONAL, BotMakerFeatures.tcoUserId,
        userInfo.getUserId());
    builder.putValue(OPTIONAL, BotMakerFeatures.spammerId,
        userInfo.getUserId());
    builder.putExtractor(OPTIONAL, BotMakerFeatures.tcoPrivacyStatus,
        new EnumExtractors.Name<>(userInfo.getPrivacyStatus()));
  }

  private void putDmUrlEventIntoBuilder(
      DmUrlEvent dmUrlEvent,
      FeatureMapBuilder builder) throws Exception {
    builder.putValue(REQUIRED, BotMakerFeatures.tcoString,
        dmUrlEvent.getTco());

    putUrlInfoIntoBuilder(dmUrlEvent.getUrlInfo(), builder);

    DmInfo dmInfo = dmUrlEvent.getDmInfo();
    builder.putValue(REQUIRED, BotMakerFeatures.timestampMs,
        dmInfo.getDmEventTimeMs());
    builder.putValue(OPTIONAL, BotMakerFeatures.dmId,
        dmInfo.getId());

    if (dmInfo.isSetDmAttributes()) {
      ExtendedDmAttributes extendedDmAttributes = dmInfo.getDmAttributes();
      builder.putValue(OPTIONAL, BotMakerFeatures.applicationId,
          extendedDmAttributes.getClientAppId());
    }
  }


  private void putUrlInfoIntoBuilder(
      UrlInfo urlInfo,
      FeatureMapBuilder builder) throws Exception {

    if (urlInfo.isSetResolution()) {
      Resolution resolution = urlInfo.getResolution();
      builder.putExtractor(OPTIONAL, BotMakerFeatures.fetchStatus,
          new EnumExtractors.Name<>(resolution.getStatus()));
      builder.putCollection(OPTIONAL, BotMakerFeatures.redirectionChain,
          resolution.getRedirectionChain(), String.class);
      builder.putCollection(OPTIONAL, BotMakerFeatures.redirectionChainIps,
          Lists.newLinkedList(Iterables.concat(resolution.getRedirectionChainIps())), String.class);
      builder.putCollection(OPTIONAL, BotMakerFeatures.originalIps,
          resolution.getOriginalIps(), String.class);
      builder.putValue(OPTIONAL, BotMakerFeatures.expiresAtMs,
          resolution.getExpiresAt());
      builder.putValue(OPTIONAL, BotMakerFeatures.lastHopFetcherError,
          resolution.getLastHopFetcherError());
      builder.putValue(OPTIONAL, BotMakerFeatures.lastHopHttpResponseStatusCode,
          resolution.getLastHopHttpResponseStatusCode());
    }

    if (urlInfo.isSetCardsMetadata()) {
      final CardsMetadata cardsMetadata = urlInfo.getCardsMetadata();
      if (cardsMetadata.isSetMetaTags()) {
        builder.putValue(
            OPTIONAL,
            BotMakerFeatures.cardMetaTags,
            cardsMetadata.getMetaTags()
        );
      }

      if (cardsMetadata.isSetResolvedImageUrls()) {
        builder.putValue(
            OPTIONAL,
            BotMakerFeatures.cardImageUrls,
            cardsMetadata.getResolvedImageUrls()
        );
      }
    }

    if (urlInfo.isSetConstructError()) {
      ConstructError constructError = urlInfo.getConstructError();
      builder.putExtractor(OPTIONAL, BotMakerFeatures.constructErrorCode,
          new EnumExtractors.Name<>(constructError.getConstructErrorCode()));
      builder.putValue(OPTIONAL, BotMakerFeatures.constructErrorString,
          constructError.getConstructErrorString());
    }
  }
}
