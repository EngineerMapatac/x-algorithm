package com.twitter.scarecrow.features;

import java.util.List;

import com.google.common.collect.ImmutableMap;

import com.twitter.relevance.feature_store.thrift.FeatureData;
import com.twitter.relevance.feature_store.thrift.FeatureValue;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureExtractionException;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.twitter_service.thorn.thriftjava.MediaData;
import com.twitter.twitter_service.thorn.thriftjava.MediaScores;
import com.twitter.twitter_service.thorn.thriftjava.PertinentInfo;
import com.twitter.twitter_service.thorn.thriftjava.ThornDistance;
import com.twitter.twitter_service.thorn.thriftjava.ThornHash;
import com.twitter.twitter_service.thorn.thriftjava.ThornInfo;
import com.twitter.twitter_service.thorn.thriftjava.ThornScore;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfThornInfo extends FeatureMapExtractor {
  private final ThornInfo event;
  private static final String SEPARATOR = "_";

  public FeaturesOfThornInfo(ThornInfo event) {
    this.event = event;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    if (event.isSet(ThornInfo._Fields.PERTINENT)) {
      PertinentInfo pertinentInfo = event.getPertinent();
      addMediaScores(pertinentInfo.getPertinentMediaScores(), builder);
      builder.putCollection(REQUIRED, BotMakerFeatures.reasons,
          pertinentInfo.getReasons(), String.class);
      builder.putCollection(REQUIRED, BotMakerFeatures.hashTypes,
          pertinentInfo.getHashTypes(), String.class);
      builder.putCollection(REQUIRED, BotMakerFeatures.matchTypes,
          pertinentInfo.getMatchTypes(), String.class);
      builder.putCollection(REQUIRED, BotMakerFeatures.sources,
          pertinentInfo.getSources(), String.class);

      ImmutableMap.Builder<String, List<Long>> distancesMap = ImmutableMap.builder();
      ImmutableMap.Builder<String, String> hashesMap = ImmutableMap.builder();

      for (ThornHash hash: pertinentInfo.hashes) {
        hashesMap.put(hash.getHashType(), hash.getHashString());
        for (ThornDistance distance: hash.getDistances()) {
          distancesMap.put(hash.getHashType() + SEPARATOR + distance.getDistanceName(),
            distance.getDistanceVector());
        }
      }

      FeatureData distancesFeature = new FeatureData()
          .setFeatureValue(FeatureValue.longListMapValue(distancesMap.build()));

      FeatureData hashesFeature = new FeatureData()
          .setFeatureValue(FeatureValue.strMapValue(hashesMap.build()));

      builder.put(REQUIRED, BotMakerFeatures.distances, distancesFeature);
      builder.put(REQUIRED, BotMakerFeatures.hashes, hashesFeature);
    } else if (event.isSet(ThornInfo._Fields.NON_PERTINENT)) {
       addMediaScores(event.getNonPertinent().getNonPertinentMediaScores(), builder);
    }
  }

  private static void addMediaScores(MediaScores mediaScores, FeatureMapBuilder builder)
      throws FeatureExtractionException {
    MediaData mediaData = mediaScores.getMediaData();
    builder.putValue(REQUIRED, BotMakerFeatures.mediaId,
        mediaData.getMediaKey().getMedia_id());
    builder.putValue(REQUIRED, BotMakerFeatures.category,
        mediaData.getMediaKey().getMedia_category().name());
    builder.putValue(REQUIRED, BotMakerFeatures.userId,
        mediaData.getUserId());
    builder.putPositive(OPTIONAL, BotMakerFeatures.tweetId,
        mediaData.getTweetId());
    builder.putPositive(OPTIONAL, BotMakerFeatures.dmId,
        mediaData.getDmId());

    ImmutableMap.Builder<String, Double> map = ImmutableMap.builder();

    for (ThornScore thornScore: mediaScores.getThornScores()) {
      map.put(thornScore.getScoreName(), thornScore.getScoreValue());
    }

    FeatureValue featureValue = new FeatureValue();
    featureValue.setDoubleMapValue(map.build());

    FeatureData featureData = new FeatureData();
    featureData.setFeatureValue(featureValue);
    builder.put(REQUIRED, BotMakerFeatures.mediaScores, featureData);
  }
}
