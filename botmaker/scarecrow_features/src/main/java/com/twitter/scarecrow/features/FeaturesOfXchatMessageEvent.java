package com.twitter.scarecrow.features;

import java.util.Optional;

import com.google.common.base.Preconditions;
import com.x.dmv2.thriftjava.MessageEvent;
import com.x.dmv2.thriftjava.MessageEventRelaySource;
import com.x.dmv2.thriftjava.MessageEventSignature;

import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;

public class FeaturesOfXchatMessageEvent extends FeatureMapExtractor {

  private final MessageEvent event;

  public FeaturesOfXchatMessageEvent(MessageEvent event) {
    this.event = Preconditions.checkNotNull(event);
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    Optional<String> sequenceIdOpt = Optional.ofNullable(event.getSequence_id());
    if (sequenceIdOpt.isPresent()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.sequenceId, sequenceIdOpt.get());
    }

    Optional<String> messageIdOpt = Optional.ofNullable(event.getMessage_id());
    if (messageIdOpt.isPresent()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.messageId, messageIdOpt.get());
    }

    Optional<Long> senderIdOpt = str2LongOpt(event.getSender_id());
    if (senderIdOpt.isPresent()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.spammerId, senderIdOpt.get());
    }

    Optional<String> conversationIdOpt = Optional.ofNullable(event.getConversation_id());
    if (conversationIdOpt.isPresent()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.conversationId, conversationIdOpt.get());
    }

    Optional<String> conversationTokenOpt = Optional.ofNullable(event.getConversation_token());
    if (conversationTokenOpt.isPresent()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.conversationToken, conversationTokenOpt.get());
    }

    Optional<Long> createdAtMsecOpt = str2LongOpt(event.getCreated_at_msec());
    if (createdAtMsecOpt.isPresent()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.eventTimestampMs, createdAtMsecOpt.get());
    }

    Optional<String> previousSequenceIdOpt = Optional.ofNullable(event.getPrevious_sequence_id());
    if (previousSequenceIdOpt.isPresent()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.previousSequenceId, previousSequenceIdOpt.get());
    }

    Optional<Boolean> isTrustedOpt = Optional.ofNullable(event.isIs_trusted());
    if (isTrustedOpt.isPresent()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.isTrusted, isTrustedOpt.get());
    }



    Optional<MessageEventRelaySource> relaySourceOpt = Optional.ofNullable(event.getRelay_source());
    if (relaySourceOpt.isPresent()) {
      builder.putValue(
          OPTIONAL,
          BotMakerFeatures.relaySource,
          relaySourceOpt.get().name()
      );
    }

    Optional<MessageEventSignature> messageEventSignatureOpt =
        Optional.ofNullable(event.getMessage_event_signature());
    if (messageEventSignatureOpt.isPresent()) {
      MessageEventSignature messageEventSignature = messageEventSignatureOpt.get();
      Optional<String> signatureOpt = Optional.ofNullable(messageEventSignature.getSignature());
      if (signatureOpt.isPresent()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.signature, signatureOpt.get());
      }

      Optional<String> publicKeyVersionOpt =
          Optional.ofNullable(messageEventSignature.getPublic_key_version());
      if (publicKeyVersionOpt.isPresent()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.publicKeyVersion, publicKeyVersionOpt.get());
      }

      Optional<String> signatureVersionOpt =
          Optional.ofNullable(messageEventSignature.getSignature_version());
      if (signatureVersionOpt.isPresent()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.signatureVersion, signatureVersionOpt.get());
      }
    }
  }

  private static Optional<Long> str2LongOpt(String s) {
    if (s == null) {
      return Optional.empty();
    } else {
      try {
        return Optional.of(Long.parseLong(s));
      } catch (NumberFormatException e) {
        return Optional.empty();
      }
    }
  }
}
