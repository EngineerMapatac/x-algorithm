package com.twitter.scarecrow.features;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.birdwatch.thriftjava.BirdwatchStatusChangeEvent;
import com.twitter.birdwatch.thriftjava.BirdwatchStatusChangeEventNoteStatusChange;
import com.twitter.birdwatch.thriftjava.BirdwatchNoteRatingStatus;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

public class FeaturesOfBirdwatchStatusChangeEvent extends FeatureMapExtractor {
  private final BirdwatchStatusChangeEvent event;

  public FeaturesOfBirdwatchStatusChangeEvent(BirdwatchStatusChangeEvent event) {
    this.event = event;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    if (event.isSetTweetId()) {
      if (event.isSetBeforeStatus() && event.isSetAfterStatus()) {
        builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.tweetId, event.getTweetId());
        boolean isPreviouslyNMR = event.getBeforeStatus()
            == BirdwatchNoteRatingStatus.NEEDS_MORE_RATINGS;
        boolean isCurrentlyCRH = event.getAfterStatus()
            == BirdwatchNoteRatingStatus.CURRENTLY_RATED_HELPFUL;
        builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.communityNoteVisible,
            isPreviouslyNMR && isCurrentlyCRH);
      } else if (event.isSetNoteStatusChanges() && event.getNoteStatusChangesSize() > 0) {
        builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.tweetId, event.getTweetId());
        boolean featureValue = false;
        for (BirdwatchStatusChangeEventNoteStatusChange change : event.getNoteStatusChanges()) {
          if (change.isSetBeforeStatus() && change.isSetAfterStatus()
              && change.getBeforeStatus() == BirdwatchNoteRatingStatus.NEEDS_MORE_RATINGS
              && change.getAfterStatus() == BirdwatchNoteRatingStatus.CURRENTLY_RATED_HELPFUL) {
            featureValue = true;
          }
        }
        builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.communityNoteVisible,
            featureValue);
      }
    }
  }
}
