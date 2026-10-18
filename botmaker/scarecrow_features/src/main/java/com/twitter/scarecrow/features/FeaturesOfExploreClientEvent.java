package com.twitter.scarecrow.features;

import java.util.ArrayList;
import java.util.List;

import com.twitter.clientapp.gen.GuideItemDetails;
import com.twitter.clientapp.gen.Item;
import com.twitter.clientapp.gen.LogEvent;
import com.twitter.guide.scribing.thriftjava.TransparentGuideDetails;
import com.twitter.guide.scribing.thriftjava.TrendUrtMetadata;
import com.twitter.logbase.gen.LogBase;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfExploreClientEvent extends FeatureMapExtractor {

  private final LogEvent logEvent;

  public FeaturesOfExploreClientEvent(LogEvent logEvent) {
    this.logEvent = logEvent;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {

    List<String> guideItemTypeList = new ArrayList<>();
    List<String> guideTokenList = new ArrayList<>();
    List<String> trendNameList = new ArrayList<>();

    if (logEvent.getEvent_details().isSetItems()) {
      for (Item item : logEvent.getEvent_details().getItems()) {
        if (item.isSetGuide_item_details()) {
          GuideItemDetails guideItemDetails = item.getGuide_item_details();

          if (item.isSetItem_type()) {
            guideItemTypeList.add(item.getItem_type().name());
          }
          if (guideItemDetails.isSetSource_data()) {
            guideTokenList.add(guideItemDetails.getSource_data());
          }

          if (guideItemDetails.isSetTransparentGuideDetails()) {
            TransparentGuideDetails transparentGuideDetails = guideItemDetails
                .getTransparentGuideDetails();

            if (transparentGuideDetails.getSetField() == TransparentGuideDetails._Fields
                .TREND_METADATA) {
              TrendUrtMetadata trendMetadata = transparentGuideDetails.getTrendMetadata();
              trendNameList.add(trendMetadata.getTrendName());
            }
          }
        }
      }

      builder.putCollection(
          REQUIRED, BotMakerFeatures.guideItemType, guideItemTypeList, String.class);
      builder.putCollection(
          REQUIRED,
          BotMakerFeatures.guideItemToken,
          guideTokenList,
          String.class
      );
      builder.putCollection(
          REQUIRED,
          BotMakerFeatures.trendName,
          trendNameList,
          String.class
      );

      if (logEvent.isSetLog_base()) {
        LogBase logBase = logEvent.getLog_base();

        if (logBase.isSetTimestamp()) {
          builder.putValue(
              REQUIRED, BotMakerFeatures.timestampMs, logBase.getTimestamp());
        }

        if (logBase.isSetUser_id()) {
          builder.putValue(
              REQUIRED, BotMakerFeatures.userId, logBase.getUser_id());
        }

        if (logBase.isSetIp_address()) {
          builder.putValue(
              REQUIRED, BotMakerFeatures.ipAddress, logBase.getIp_address());
        }
      }
    }
  }
}
