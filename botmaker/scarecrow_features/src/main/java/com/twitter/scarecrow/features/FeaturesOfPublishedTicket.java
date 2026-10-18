package com.twitter.scarecrow.features;

import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import com.google.common.base.Preconditions;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureExtractionException;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.ticketservice.thriftjava.PropertyBag;
import com.twitter.ticketservice.thriftjava.PublishedTicket;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfPublishedTicket extends FeatureMapExtractor {

  private final PublishedTicket ticket;

  public FeaturesOfPublishedTicket(PublishedTicket ticket) {
    this.ticket = Preconditions.checkNotNull(ticket);
  }

  private void addFeaturesFromPropertyBag(
      FeatureMapBuilder builder,
      @Nullable PropertyBag bag,
      String prefix
  ) throws FeatureExtractionException {
    if (bag != null) {
      if (bag.isSetStringEntries()) {
        for (Map.Entry<String, String> entry : bag.getStringEntries().entrySet()) {
          builder.putValue(OPTIONAL, prefix + entry.getKey(), entry.getValue());
        }
      }

      if (bag.isSetStringListEntries()) {
        for (Map.Entry<String, List<String>> entry : bag.getStringListEntries().entrySet()) {
          builder.putCollection(OPTIONAL, prefix + entry.getKey(), entry.getValue(), String.class);
        }
      }
    }
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(REQUIRED, BotMakerFeatures.eventId, ticket.getTicketId());

    builder.putValue(OPTIONAL, BotMakerFeatures.timestampMs, ticket.getStartTimeMs());
    builder.putValue(OPTIONAL, BotMakerFeatures.loggedInUserId, ticket.getLoggedInUserId());

    addFeaturesFromPropertyBag(builder, ticket.getUser(), "user");
    addFeaturesFromPropertyBag(builder, ticket.getTicket(), "ticket");
    addFeaturesFromPropertyBag(builder, ticket.getScreenNameUser(), "screenNameUser");
  }
}
