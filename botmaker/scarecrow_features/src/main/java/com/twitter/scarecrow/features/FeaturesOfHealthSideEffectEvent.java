package com.twitter.scarecrow.features;

import java.util.Map;

import com.fasterxml.jackson.databind.ObjectMapper;

import com.twitter.hmli.health_side_effects.botmaker.thriftjava.BotmakerSideEffect;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.BotmakerSideEffectData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.CompromiseData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.JsonObjectData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.MisinfoApeAttReportData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.MisinfoApeSendInternalEmailData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.NsfwCommunityData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.NsfwTweetMediaData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.NsfwUserData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.ProactiveTosData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.ReactiveTosData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.SignupData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.SpammyTweetContentData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.SpokScoreData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.GroxScoreData;
import com.twitter.hmli.health_side_effects.botmaker.thriftjava.GroxUserScoreData;
import com.twitter.mediaservices.commons.thriftscala.MediaCategory;
import com.twitter.mediaservices.media_util.GenericMediaKey;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;
import static com.twitter.botmaker.FeatureModifier.OPTIONAL;

public class FeaturesOfHealthSideEffectEvent extends FeatureMapExtractor {
  private final BotmakerSideEffect event;

  public FeaturesOfHealthSideEffectEvent(BotmakerSideEffect event) {
    this.event = event;
  }

  private void extractCompromiseData(BotmakerSideEffectData data, FeatureMapBuilder builder)
      throws Exception {
    CompromiseData pCompromise = data.getPCompromise();
    builder.putValue(REQUIRED, BotMakerFeatures.userId, pCompromise.getUserId());
    if (pCompromise.isSetRawScore()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.rawScore,
                       pCompromise.getRawScore());
    }
  }

  private void extractSpammyTweetContent(BotmakerSideEffectData data, FeatureMapBuilder builder)
      throws Exception {
    SpammyTweetContentData spammyTweetContentData = data.getSpammyTweetContent();
    builder.putValue(REQUIRED, BotMakerFeatures.tweetId, spammyTweetContentData.getTweetId());
    builder.putValue(REQUIRED, BotMakerFeatures.scoreMap,
        spammyTweetContentData.getScoreByModelName());
  }

  private void extractSignup(BotmakerSideEffectData data, FeatureMapBuilder builder)
      throws Exception {
    SignupData signup = data.getPSignup();
    builder.putValue(REQUIRED, BotMakerFeatures.userId, signup.getUserId());
    builder.putValue(REQUIRED, BotMakerFeatures.timestampMs, signup.getUc10TimestampMs());
    builder.putValue(REQUIRED, BotMakerFeatures.predictTimestampMs,
        signup.getPredictTimestampMs());
    builder.putValue(REQUIRED, BotMakerFeatures.scoreMap, signup.getScoreByModelName());
  }

  private void extractReactiveTos(BotmakerSideEffectData data, FeatureMapBuilder builder)
      throws Exception {
    ReactiveTosData reactiveTos = data.getReactiveTos();
    builder.putValue(REQUIRED, BotMakerFeatures.rawScore, reactiveTos.getRawScore());
    builder.putValue(REQUIRED, BotMakerFeatures.calibratedScore, reactiveTos.getCalibratedScore());
    if (reactiveTos.isSetCaseUpdateTimeMs()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.caseUpdateTimeMs,
                       reactiveTos.getCaseUpdateTimeMs());
    }
    if (reactiveTos.isSetCaseKey()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.caseKey, reactiveTos.getCaseKey());
    }
    if (reactiveTos.isSetCaseUpdateId()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.caseUpdateId, reactiveTos.getCaseUpdateId());
    }
    if (reactiveTos.isSetEvidenceKey()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.evidenceKey, reactiveTos.getEvidenceKey());
    }
    if (reactiveTos.isSetUpdateType()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.caseUpdateType, reactiveTos.getUpdateType());
    }
    if (reactiveTos.isSetReportKey()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.reportKey, reactiveTos.getReportKey());
    }
  }

  private void extractProactiveTos(BotmakerSideEffectData data, FeatureMapBuilder builder)
      throws Exception {
    ProactiveTosData proactiveTos = data.getProactiveTos();
    builder.putValue(REQUIRED, BotMakerFeatures.tweetId, proactiveTos.getTweetId());

    Map<String, Map<String, Double>> scoresByLabel = proactiveTos.getScoresByLabel();
    ObjectMapper objectMapper = new ObjectMapper();
    String scoreMap = objectMapper.writeValueAsString(scoresByLabel);
    builder.putValue(REQUIRED, BotMakerFeatures.scoreMap, scoreMap);

    builder.putValue(REQUIRED, BotMakerFeatures.predictTimestampMs,
        proactiveTos.getPredictTimestampMs());
    builder.putValue(REQUIRED, BotMakerFeatures.modelName, proactiveTos.getModelName());
    builder.putValue(REQUIRED, BotMakerFeatures.modelVersion, proactiveTos.getModelVersion());
    builder.putValue(REQUIRED, BotMakerFeatures.language, proactiveTos.getLang());
  }

  private void extractJsonObject(BotmakerSideEffectData data, FeatureMapBuilder builder)
      throws Exception {
    JsonObjectData jsonObject = data.getJsonObject();

    builder.putValue(REQUIRED, BotMakerFeatures.eventTimestampMs, jsonObject.getEventTimestampMs());
    builder.putValue(REQUIRED, BotMakerFeatures.jsonObjectName, jsonObject.getObjectName());
    builder.putValue(REQUIRED, BotMakerFeatures.jsonObjectVersion, jsonObject.getObjectVersion());
    builder.putValue(REQUIRED, BotMakerFeatures.jsonObjectStr, jsonObject.getJsonStr());
  }

  private void extractNsfwUserData(BotmakerSideEffectData data, FeatureMapBuilder builder)
      throws Exception {
    NsfwUserData nsfwUserData = data.getNsfwUser();
    builder.putValue(REQUIRED, BotMakerFeatures.userId, nsfwUserData.getUserId());
    builder.putValue(REQUIRED, BotMakerFeatures.modelName, nsfwUserData.getModelName());
  }

  private void extractNsfwCommunityData(BotmakerSideEffectData data, FeatureMapBuilder builder)
      throws Exception {
    NsfwCommunityData nsfwCommunityData = data.getNsfwCommunity();
    builder.putValue(REQUIRED, BotMakerFeatures.communityId, nsfwCommunityData.getCommunityId());
  }

  private void extractMisinfoApeAttReportData(BotmakerSideEffectData data,
    FeatureMapBuilder builder)
      throws Exception {
    MisinfoApeAttReportData misinfoApeAttReportData =
      data.getMisinfoApeAttReport();
    builder.putValue(REQUIRED, BotMakerFeatures.tweetId,
      misinfoApeAttReportData.getTweetId());
    builder.putValue(REQUIRED, BotMakerFeatures.userId,
      misinfoApeAttReportData.getUserId());
    builder.putValue(REQUIRED, BotMakerFeatures.botId, misinfoApeAttReportData.getBotId());
  }

  private void extractMisinfoApeSendInternalEmailData(BotmakerSideEffectData data,
    FeatureMapBuilder builder)
      throws Exception {
    MisinfoApeSendInternalEmailData misinfoApeSendInternalEmailData =
      data.getMisinfoApeSendInternalEmail();
    builder.putValue(REQUIRED, BotMakerFeatures.recipients,
      misinfoApeSendInternalEmailData.getRecipients());
    builder.putValue(REQUIRED, BotMakerFeatures.subject,
      misinfoApeSendInternalEmailData.getSubject());
    builder.putValue(REQUIRED, BotMakerFeatures.body, misinfoApeSendInternalEmailData.getBody());
  }

  private void extractNsfwTweetMediaData(BotmakerSideEffectData data, FeatureMapBuilder builder)
      throws Exception {
    NsfwTweetMediaData nsfwTweetMediaData = data.getNsfwTweetMedia();
    builder.putValue(REQUIRED, BotMakerFeatures.tweetId, nsfwTweetMediaData.getTweetId());
    builder.putValue(REQUIRED, BotMakerFeatures.userId, nsfwTweetMediaData.getUserId());
    GenericMediaKey genericMediaKey = GenericMediaKey.apply(
        MediaCategory.apply(nsfwTweetMediaData.getMediaKey().getMedia_category().getValue()),
        nsfwTweetMediaData.getMediaKey().getMedia_id(),
        null
    );
    builder.putValue(REQUIRED, BotMakerFeatures.genericMediaKey, genericMediaKey.toStringKey());
    builder.putValue(REQUIRED, BotMakerFeatures.source, nsfwTweetMediaData.getSource());
    if (nsfwTweetMediaData.isSetLabel()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.label, nsfwTweetMediaData.getLabel());
    }
    if (nsfwTweetMediaData.isSetScore()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.score, nsfwTweetMediaData.getScore());
    }
  }

  private void extractSpokScoreData(BotmakerSideEffectData data, FeatureMapBuilder builder)
      throws Exception {
    SpokScoreData spokScoreData = data.getSpokScore();
    builder.putValue(REQUIRED, BotMakerFeatures.userId, spokScoreData.getUserId());
    builder.putValue(REQUIRED, BotMakerFeatures.pSpam, spokScoreData.getPSpam());
    builder.putValue(REQUIRED, BotMakerFeatures.pNotSpam, spokScoreData.getPNotSpam());
    builder.putValue(REQUIRED, BotMakerFeatures.createdAtMs, spokScoreData.getCreatedAtMsec());
    builder.putValue(REQUIRED, BotMakerFeatures.lastReplyIds, spokScoreData.getLastReplyIds());
  }

  private void extractGroxScoreData(BotmakerSideEffectData data, FeatureMapBuilder builder)
      throws Exception {
    GroxScoreData groxScoreData = data.getGroxScore();
    builder.putValue(REQUIRED, BotMakerFeatures.userId, groxScoreData.getUserId());
    builder.putValue(REQUIRED, BotMakerFeatures.postId, groxScoreData.getPostId());
    builder.putValue(REQUIRED, BotMakerFeatures.createdAt, groxScoreData.getCreatedAt());
    builder.putValue(REQUIRED, BotMakerFeatures.spamScore, groxScoreData.getSpamScore());
  }

  private void extractGroxUserScoreData(BotmakerSideEffectData data, FeatureMapBuilder builder)
      throws Exception {
    GroxUserScoreData groxUserScoreData = data.getGroxUserScore();
    builder.putValue(REQUIRED, BotMakerFeatures.userId, groxUserScoreData.getUserId());
    builder.putValue(REQUIRED, BotMakerFeatures.createdAt, groxUserScoreData.getCreatedAt());
    builder.putValue(REQUIRED, BotMakerFeatures.spamScore, groxUserScoreData.getSpamScore());
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(REQUIRED, BotMakerFeatures.healthSideEffectSubEvent, event.getSubeventName());
    BotmakerSideEffectData data = event.getData();
    switch (data.getSetField()) {
      case P_SIGNUP:
        extractSignup(data, builder);
        break;
      case REACTIVE_TOS:
        extractReactiveTos(data, builder);
        break;
      case PROACTIVE_TOS:
        extractProactiveTos(data, builder);
        break;
      case JSON_OBJECT:
        extractJsonObject(data, builder);
        break;
      case P_COMPROMISE:
        extractCompromiseData(data, builder);
        break;
      case NSFW_USER:
        extractNsfwUserData(data, builder);
        break;
      case SPAMMY_TWEET_CONTENT:
        extractSpammyTweetContent(data, builder);
        break;
      case NSFW_COMMUNITY:
        extractNsfwCommunityData(data, builder);
        break;
      case MISINFO_APE_ATT_REPORT:
        extractMisinfoApeAttReportData(data, builder);
        break;
      case MISINFO_APE_SEND_INTERNAL_EMAIL:
        extractMisinfoApeSendInternalEmailData(data, builder);
        break;
      case NSFW_TWEET_MEDIA:
        extractNsfwTweetMediaData(data, builder);
        break;
      case SPOK_SCORE:
        extractSpokScoreData(data, builder);
        break;
      case GROX_SCORE:
        extractGroxScoreData(data, builder);
        break;
      case GROX_USER_SCORE:
        extractGroxUserScoreData(data, builder);
        break;
      default:
        break;
    }
  }
}
