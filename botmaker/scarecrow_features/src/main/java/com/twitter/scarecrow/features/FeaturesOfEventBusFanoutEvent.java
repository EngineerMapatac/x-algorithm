package com.twitter.scarecrow.features;

import java.nio.ByteBuffer;
import java.util.List;
import java.util.Map;
import java.util.Set;

import com.google.common.base.Optional;
import com.google.common.base.Preconditions;
import com.google.common.collect.Lists;
import com.google.common.collect.Sets;

import com.twitter.accounts.util.CryptoUtils;
import com.twitter.convosvc.thriftjava.ClearConversation;
import com.twitter.convosvc.thriftjava.EventBusFanoutEvent;
import com.twitter.convosvc.thriftjava.EventType;
import com.twitter.convosvc.thriftjava.JoinConversation;
import com.twitter.convosvc.thriftjava.LeaveConversation;
import com.twitter.convosvc.thriftjava.MessageCreate;
import com.twitter.convosvc.thriftjava.ScarecrowRequestInfo;
import com.twitter.convosvc.thriftjava.TrustConversation;
import com.twitter.dmservice.thrift.Dm;
import com.twitter.dmservice.thrift.HashtagEntity;
import com.twitter.service.gen.scarecrow.DmParticipants;
import com.twitter.service.gen.scarecrow.DmPrivilege;
import com.twitter.spam.botmaker_features.AppIdExtractors;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureExtractionException;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfEventBusFanoutEvent extends FeatureMapExtractor {

    private final EventBusFanoutEvent fanoutEvent;

    public FeaturesOfEventBusFanoutEvent(EventBusFanoutEvent fanoutEvent) {
        this.fanoutEvent = Preconditions.checkNotNull(fanoutEvent);
    }

    @Override
    public void apply(FeatureMapBuilder builder) throws Exception {
        final EventType eventType = this.fanoutEvent.getEvent().getEvent_type();

        if (eventType == EventType.MESSAGE_CREATE) {
            final MessageCreate messageCreateEvent =
                this.fanoutEvent.getEvent().getEvent_detail().getMessage_create();
            final Dm dm = messageCreateEvent.getDm();

            if (dm == null) {
                throw new NullPointerException(String.format("missing DM: %s", messageCreateEvent));
            }

            builder
                .putPositive(REQUIRED, BotMakerFeatures.actorId, dm.getSender_id())
                .putPositive(REQUIRED, BotMakerFeatures.spammerId, dm.getSender_id())
                .putPositive(OPTIONAL, BotMakerFeatures.victimId, dm.getRecipient_id())
                .putPositive(OPTIONAL, BotMakerFeatures.conversationId, dm.getConversation_id())
                .putValue(REQUIRED, BotMakerFeatures.dmText, dm.getText())
                .putPositive(REQUIRED, BotMakerFeatures.eventId, dm.getId())
                .putValue(OPTIONAL, BotMakerFeatures.dmCreatedVia, dm.getCreated_via())
                .putExtractor(OPTIONAL,
                    BotMakerFeatures.applicationId, AppIdExtractors.fromDirectMessage(dm));

            if (dm.isSetHashtags()) {
                processHashtagEntities(builder, dm.getHashtags());
            }

            if (fanoutEvent.isSetScarecrow_info()) {
                ScarecrowRequestInfo scarecrowInfo = fanoutEvent.getScarecrow_info();

                if (scarecrowInfo.isSet(ScarecrowRequestInfo._Fields.DIRECT_MESSAGE)
                    && scarecrowInfo.getDirect_message() != null) {
                    if (scarecrowInfo.getDirect_message().isSetFirstHopUrls()) {
                        builder.putCollection(OPTIONAL,
                            BotMakerFeatures.firstHopUrls,
                            scarecrowInfo.getDirect_message().getFirstHopUrls(), String.class);
                    }

                    if (scarecrowInfo.getDirect_message().isSetPrivileges()) {
                        Set<String> privilegeNames = Sets.newHashSet();
                        for (DmPrivilege privilege
                            : scarecrowInfo.getDirect_message().getPrivileges()) {
                            privilegeNames.add(privilege.name());
                        }
                        builder.putCollection(OPTIONAL,
                            BotMakerFeatures.privileges, privilegeNames, String.class);
                    }
                }
            }

            if (messageCreateEvent.isSetIs_low_quality()) {
                builder
                    .putValue(OPTIONAL,
                        BotMakerFeatures.isLowQuality, messageCreateEvent.isIs_low_quality());
            }
        } else if (eventType == EventType.JOIN_CONVERSATION) {
            final JoinConversation joinConvEvent =
                this.fanoutEvent.getEvent().getEvent_detail().getJoin_conversation();

            builder
                .putPositive(OPTIONAL,
                    BotMakerFeatures.actorId, joinConvEvent.getInitiating_user_id())
                .putPositive(OPTIONAL,
                    BotMakerFeatures.spammerId, joinConvEvent.getInitiating_user_id())
                .putCollection(OPTIONAL,
                    BotMakerFeatures.victimIds,
                    joinConvEvent.getParticipants_snapshot(), Long.class);

            if (joinConvEvent.isSetIs_low_quality()) {
                builder
                    .putValue(OPTIONAL,
                        BotMakerFeatures.isLowQuality, joinConvEvent.isIs_low_quality());
            }
        } else if (eventType == EventType.CLEAR_CONVERSATION) {
            final ClearConversation clearConversationEvent =
                this.fanoutEvent.getEvent().getEvent_detail().getClear_conversation();

            if (clearConversationEvent.isSetBlocked_user_id()) {
                builder
                    .putValue(OPTIONAL,
                        BotMakerFeatures.blockedUserId,
                        clearConversationEvent.getBlocked_user_id());
            }

            if (clearConversationEvent.isSetUntrusted_user_id()) {
                builder
                    .putValue(OPTIONAL,
                        BotMakerFeatures.untrustedUserId,
                        clearConversationEvent.getUntrusted_user_id());
            }
        } else if (eventType == EventType.TRUST_CONVERSATION) {
            final TrustConversation trustConversationEvent =
                this.fanoutEvent.getEvent().getEvent_detail().getTrust_conversation();

            if (trustConversationEvent.isSetTrusted_user_id()) {
                builder
                    .putValue(OPTIONAL,
                        BotMakerFeatures.trustedUserId,
                        trustConversationEvent.getTrusted_user_id());
            }
        } else if (eventType == EventType.LEAVE_CONVERSATION) {
            final LeaveConversation leaveConversationEvent =
                this.fanoutEvent.getEvent().getEvent_detail().getLeave_conversation();

            if (leaveConversationEvent.isSetUntrusted_user_id()) {
                builder
                    .putValue(OPTIONAL,
                        BotMakerFeatures.untrustedUserId,
                        leaveConversationEvent.getUntrusted_user_id());
            }
        }

        if (fanoutEvent.isSetContext()) {
            builder.putValue(OPTIONAL,
                BotMakerFeatures.auditIp, fanoutEvent.getContext().getAudit_ip());

            if (fanoutEvent.getContext().isSetKnown_device_token()) {
                builder.putValue(OPTIONAL,
                    BotMakerFeatures.knownDeviceToken,
                    fanoutEvent.getContext().getKnown_device_token());
            }
            if (fanoutEvent.getContext().isSetSession_hash()) {
                builder.putValue(OPTIONAL,
                    BotMakerFeatures.sessionHash,
                        CryptoUtils.decode(ByteBuffer.wrap(
                            fanoutEvent.getContext().getSession_hash())));
            }
            if (fanoutEvent.getContext().isSetUser_agent()) {
                builder.putValue(OPTIONAL,
                    BotMakerFeatures.userAgent,
                    fanoutEvent.getContext().getUser_agent());
            }
            if (fanoutEvent.getContext().isSetBotox_validation_status()) {
                builder.putValue(OPTIONAL,
                    BotMakerFeatures.botoxValidationStatus,
                    fanoutEvent.getContext().getBotox_validation_status());
            }
            if (fanoutEvent.getContext().isSetApp_attestation_status()) {
                builder.putValue(OPTIONAL,
                    BotMakerFeatures.appAttestationStatus,
                    fanoutEvent.getContext().getApp_attestation_status());
            }
            if (fanoutEvent.getContext().isSetConnection_fingerprint()) {
                builder.putValue(OPTIONAL,
                    BotMakerFeatures.connectionFingerprint,
                    fanoutEvent.getContext().getConnection_fingerprint());
            }
        }

        builder.putValue(OPTIONAL, BotMakerFeatures.dmConversationId,
            this.fanoutEvent.getEvent().getConversation_id());

        if (fanoutEvent.isSetFanout_user_ids() && fanoutEvent.getFanout_user_ids().size() > 0) {
            builder.putCollection(OPTIONAL, BotMakerFeatures.participantIds,
                fanoutEvent.getFanout_user_ids(), Long.class);
        }

        Optional<DmParticipants> participants = getDmParticipants(fanoutEvent);
        if (participants.isPresent()) {
            addPrivilegesToBuilder(builder, participants.get());
        }
    }

    private void processHashtagEntities(FeatureMapBuilder builder, List<HashtagEntity> hashtags)
        throws FeatureExtractionException {
        List<String> texts = Lists.newArrayListWithCapacity(hashtags.size());

        for (HashtagEntity hashtag : hashtags) {
            if (hashtag.isSetText()) {
                texts.add(hashtag.getText());
            }
        }

        builder.putCollection(OPTIONAL, BotMakerFeatures.hashtags, texts, String.class);
    }

    protected Optional<DmParticipants> getDmParticipants(EventBusFanoutEvent eventBusFanoutEvent) {
        if (eventBusFanoutEvent.isSetScarecrow_info()) {
            ScarecrowRequestInfo scarecrowInfo = eventBusFanoutEvent.getScarecrow_info();

            if (scarecrowInfo.isSet(ScarecrowRequestInfo._Fields.CONVERSATION_CREATE)
                && scarecrowInfo.getConversation_create() != null
                && scarecrowInfo.getConversation_create().isSetParticipants()) {
                return Optional.of(scarecrowInfo.getConversation_create().getParticipants());
            }
        }
        return Optional.absent();
    }

    protected void addUserToPrivilegeSet(Set<DmPrivilege> privileges, DmPrivilege privilege,
                                         Long userId, Set<Long> setToAdd) {
        if (privileges.contains(privilege)) {
            setToAdd.add(userId);
        }
    }

    protected void addLongSetToBuilder(Set<Long> set, FeatureMapBuilder builder, String featureName)
        throws Exception {
        if (set.size() > 0) {
            builder.putCollection(OPTIONAL, featureName, set, Long.class);
        }
    }

    protected void addPrivilegesToBuilder(FeatureMapBuilder builder, DmParticipants participants)
        throws Exception {
        Set<Long> dmPrivilegeRecipientAllowsDmFromAnyone = Sets.newHashSet();
        Set<Long> dmPrivilegeRecipientAllowsDmFromVerified = Sets.newHashSet();
        Set<Long> dmPrivilegeRecipientFollowsSender = Sets.newHashSet();
        Set<Long> dmPrivilegeRecipientTrustsSender = Sets.newHashSet();
        Set<Long> dmPrivilegeSenderCanDmAnyone = Sets.newHashSet();
        Set<Long> dmPrivilegeSentToExistingConversation = Sets.newHashSet();
        for (Map.Entry<Long, Set<DmPrivilege>> entry : participants.getPrivileges().entrySet()) {
            Long userId = entry.getKey();
            Set<DmPrivilege> userPrivileges = entry.getValue();
            addUserToPrivilegeSet(userPrivileges, DmPrivilege.RECIPIENT_ALLOWS_DMS_FROM_ANYONE,
                userId, dmPrivilegeRecipientAllowsDmFromAnyone);
            addUserToPrivilegeSet(userPrivileges, DmPrivilege.RECIPIENT_ALLOWS_DMS_FROM_VERIFIED,
                userId, dmPrivilegeRecipientAllowsDmFromVerified);
            addUserToPrivilegeSet(userPrivileges, DmPrivilege.RECIPIENT_FOLLOWS_SENDER,
                userId, dmPrivilegeRecipientFollowsSender);
            addUserToPrivilegeSet(userPrivileges, DmPrivilege.RECIPIENT_TRUSTS_SENDER,
                userId, dmPrivilegeRecipientTrustsSender);
            addUserToPrivilegeSet(userPrivileges, DmPrivilege.SENT_TO_EXISTING_CONVERSATION,
                userId, dmPrivilegeSentToExistingConversation);
            addUserToPrivilegeSet(userPrivileges, DmPrivilege.SENDER_CAN_DM_ANYONE,
                userId, dmPrivilegeSenderCanDmAnyone);
        }

        addLongSetToBuilder(dmPrivilegeRecipientAllowsDmFromAnyone, builder,
            BotMakerFeatures.dmPrivilegeRecipientAllowsDmFromAnyone);
        addLongSetToBuilder(dmPrivilegeRecipientAllowsDmFromVerified, builder,
            BotMakerFeatures.dmPrivilegeRecipientAllowsDmFromVerified);
        addLongSetToBuilder(dmPrivilegeRecipientFollowsSender, builder,
            BotMakerFeatures.dmPrivilegeRecipientFollowsSender);
        addLongSetToBuilder(dmPrivilegeRecipientTrustsSender, builder,
            BotMakerFeatures.dmPrivilegeRecipientTrustsSender);
        addLongSetToBuilder(dmPrivilegeSenderCanDmAnyone, builder,
            BotMakerFeatures.dmPrivilegeSenderCanDmAnyone);
        addLongSetToBuilder(dmPrivilegeSentToExistingConversation, builder,
            BotMakerFeatures.dmPrivilegeSentToExistingConversation);
    }
}
