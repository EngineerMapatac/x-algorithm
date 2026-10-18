package com.twitter.scarecrow.features;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

import com.google.common.base.Preconditions;
import com.google.common.collect.Lists;
import com.google.common.collect.Maps;

import com.twitter.relevance.feature_store.thrift.FeatureData;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureExtractionHelper;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import com.twitter.useng.rtp.core.thriftjava.CaseMetaData;
import com.twitter.useng.rtp.core.thriftjava.CaseUpdate;
import com.twitter.useng.rtp.core.thriftjava.CaseUpdateEvent;
import com.twitter.useng.rtp.core.thriftjava.Entity;
import com.twitter.useng.rtp.core.thriftjava.Evidence;
import com.twitter.useng.rtp.core.thriftjava.EvidenceRank;
import com.twitter.useng.rtp.core.thriftjava.Policy;
import com.twitter.useng.rtp.core.thriftjava.Report;
import com.twitter.useng.rtp.core.thriftjava.SupportCase;

import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfRtpCaseUpdateEvent extends FeatureMapExtractor {

  private final CaseUpdateEvent caseUpdateEvent;

  public FeaturesOfRtpCaseUpdateEvent(CaseUpdateEvent caseUpdateEvent) {
    this.caseUpdateEvent = Preconditions.checkNotNull(caseUpdateEvent);
  }

  interface ReportContext {
    Report getReport();
  }

  interface EvidencesContext {
    List<Evidence> getEvidences();
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    SupportCase supportCase = caseUpdateEvent.supportCase;
    CaseMetaData caseMetaData = supportCase.caseMetaData;
    builder.putValue(REQUIRED, BotMakerFeatures.caseKey, supportCase.caseKey);
    builder.putValue(REQUIRED, BotMakerFeatures.tool, caseMetaData.tool.name());
    builder.putValue(REQUIRED, BotMakerFeatures.caseGroupId, caseMetaData.caseGroupId);
    builder.putValue(REQUIRED, BotMakerFeatures.language,
        caseMetaData.language.getValue());
    builder.putValue(REQUIRED, BotMakerFeatures.caseState, supportCase.caseState.name());
    builder.putValue(REQUIRED, BotMakerFeatures.supportCaseUniqueEntityId,
        caseMetaData.uniqueEntityId.entityId);
    builder.putValue(REQUIRED, BotMakerFeatures.supportCaseUniqueEntityType,
        caseMetaData.uniqueEntityId.entityType.name());

    if (supportCase.isSetAgentId()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.agentId, supportCase.agentId);
    }
    if (supportCase.isSetQueueId()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.queueId, supportCase.queueId);
    }
    if (supportCase.isSetCasePriority()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.priority, supportCase.casePriority.priority);
    }

    Map<String, FeatureData> thriftFeatures =
        FeatureExtractionHelper.toFeatureMap(caseUpdateEvent, true);
    for (Map.Entry<String, FeatureData> entry : thriftFeatures.entrySet()) {
      builder.put(OPTIONAL, entry.getKey(), entry.getValue());
    }

    if (caseUpdateEvent.isSetCaseUpdates()) {
      List<CaseUpdate> caseUpdates = caseUpdateEvent.getCaseUpdates();
      List<Map<String, String>> caseUpdateList = Lists.newArrayList();
      Map<String, Map<String, String>> evidenceKeyToEvidenceInfo = Maps.newLinkedHashMap();

      for (CaseUpdate caseUpdate : caseUpdates) {
        Map<String, String> caseUpdateToInfo = Maps.newLinkedHashMap();
        caseUpdateList.add(caseUpdateToInfo);
        caseUpdateToInfo.put(BotMakerFeatures.caseUpdateType,
            String.valueOf(caseUpdate.getSetField().getFieldName()));

        if (isAutoCloseCaseUpdate(caseUpdate)
            && caseUpdate.getAutoCloseCaseUpdate().isSetPreviousCaseState()) {
          caseUpdateToInfo.put(BotMakerFeatures.previousCaseState,
              caseUpdate.getAutoCloseCaseUpdate().previousCaseState.name());
        }

        if (isAutoCloseCaseUpdate(caseUpdate)) {
          if (caseUpdate.getAutoCloseCaseUpdate().getNote() != null) {
            caseUpdateToInfo.put(BotMakerFeatures.note,
                caseUpdate.getAutoCloseCaseUpdate().getNote());
          }
        } else if (isCloseCaseUpdate(caseUpdate)) {
          if (caseUpdate.getCloseCaseUpdate().getNote() != null) {
            caseUpdateToInfo.put(BotMakerFeatures.note,
                caseUpdate.getCloseCaseUpdate().getNote());
          }
        }

        if (caseUpdate.isSet(CaseUpdate._Fields.NEW_CASE)
            || caseUpdate.isSet(CaseUpdate._Fields.NEW_REPORT)
            || caseUpdate.isSet(CaseUpdate._Fields.NEW_EVIDENCE)
            || caseUpdate.isSet(CaseUpdate._Fields.AUTO_CLOSE_CASE_UPDATE)
            || caseUpdate.isSet(CaseUpdate._Fields.CLOSE_CASE_UPDATE)
            || caseUpdate.isSet(CaseUpdate._Fields.EVIDENCE_RANK_UPDATE)
        ) {
          List<Map<String, String>> listOfReports = processReportFields(caseUpdate);
          for (Map<String, String> reportMap : listOfReports) {
            if (reportMap.containsKey(BotMakerFeatures.reporterType)) {
              caseUpdateToInfo.put(BotMakerFeatures.reporterType,
                  reportMap.get(BotMakerFeatures.reporterType));
            }
            if (reportMap.containsKey(BotMakerFeatures.reporterId)) {
              caseUpdateToInfo.put(BotMakerFeatures.reporterId,
                  reportMap.get(BotMakerFeatures.reporterId));
            }
            if (reportMap.containsKey(BotMakerFeatures.reportedUserId)) {
              caseUpdateToInfo.put(BotMakerFeatures.reportedUserId,
                  reportMap.get(BotMakerFeatures.reportedUserId));
            }
          }

          List<Map<String, String>> listOfEvidences = processEvidenceFields(caseUpdate);
          for (Map<String, String> evidenceMap : listOfEvidences) {
            Map<String, String> evidenceInfoMap = evidenceKeyToEvidenceInfo.computeIfAbsent(
                evidenceMap.get(BotMakerFeatures.evidenceKeys),
                k -> Maps.newLinkedHashMap()
            );

            if (evidenceMap.containsKey(BotMakerFeatures.tweetId)) {
              caseUpdateToInfo.put(BotMakerFeatures.tweetId,
                  evidenceMap.get(BotMakerFeatures.tweetId));
              evidenceInfoMap.put(
                  BotMakerFeatures.tweetId,
                  evidenceMap.get(BotMakerFeatures.tweetId)
              );
            }
            if (evidenceMap.containsKey(BotMakerFeatures.userId)) {
              caseUpdateToInfo.put(BotMakerFeatures.userId,
                  evidenceMap.get(BotMakerFeatures.userId));
              evidenceInfoMap.put(
                  BotMakerFeatures.userId,
                  evidenceMap.get(BotMakerFeatures.userId)
              );
            }
            if (evidenceMap.containsKey(BotMakerFeatures.evidenceActions)) {
              caseUpdateToInfo.put(BotMakerFeatures.evidenceActions,
                  evidenceMap.get(BotMakerFeatures.evidenceActions));
              evidenceInfoMap.put(
                  BotMakerFeatures.evidenceActions,
                  evidenceMap.get(BotMakerFeatures.evidenceActions)
              );
            }
            if (evidenceMap.containsKey(BotMakerFeatures.evidenceActionPolicyTags)) {
              caseUpdateToInfo.put(BotMakerFeatures.evidenceActionPolicyTags,
                  evidenceMap.get(BotMakerFeatures.evidenceActionPolicyTags));
              evidenceInfoMap.put(
                  BotMakerFeatures.evidenceActionPolicyTags,
                  evidenceMap.get(BotMakerFeatures.evidenceActionPolicyTags)
              );
            }
            if (evidenceMap.containsKey(BotMakerFeatures.modelType)) {
              caseUpdateToInfo.put(BotMakerFeatures.modelType,
                  evidenceMap.get(BotMakerFeatures.modelType));
              evidenceInfoMap.put(
                  BotMakerFeatures.modelType,
                  evidenceMap.get(BotMakerFeatures.modelType)
              );
            }
            if (evidenceMap.containsKey(BotMakerFeatures.modelVersion)) {
              caseUpdateToInfo.put(BotMakerFeatures.modelVersion,
                  evidenceMap.get(BotMakerFeatures.modelVersion));
              evidenceInfoMap.put(
                  BotMakerFeatures.modelVersion,
                  evidenceMap.get(BotMakerFeatures.modelVersion)
              );
            }
            if (evidenceMap.containsKey(BotMakerFeatures.rank)) {
              caseUpdateToInfo.put(BotMakerFeatures.rank,
                  evidenceMap.get(BotMakerFeatures.rank));
              evidenceInfoMap.put(
                  BotMakerFeatures.rank,
                  evidenceMap.get(BotMakerFeatures.rank)
              );
            }
            if (evidenceMap.containsKey(BotMakerFeatures.score)) {
              caseUpdateToInfo.put(BotMakerFeatures.score,
                  evidenceMap.get(BotMakerFeatures.score));
              evidenceInfoMap.put(
                  BotMakerFeatures.score,
                  evidenceMap.get(BotMakerFeatures.score)
              );
            }
            if (evidenceMap.containsKey(BotMakerFeatures.updatedTimestampMs)) {
              caseUpdateToInfo.put(BotMakerFeatures.updatedTimestampMs,
                  evidenceMap.get(BotMakerFeatures.updatedTimestampMs));
              evidenceInfoMap.put(
                  BotMakerFeatures.updatedTimestampMs,
                  evidenceMap.get(BotMakerFeatures.updatedTimestampMs)
              );
            }
            if (evidenceMap.containsKey(BotMakerFeatures.evidenceEntity)) {
              caseUpdateToInfo.put(BotMakerFeatures.evidenceEntity,
                  evidenceMap.get(BotMakerFeatures.evidenceEntity));
              evidenceInfoMap.put(
                  BotMakerFeatures.evidenceEntity,
                  evidenceMap.get(BotMakerFeatures.evidenceEntity)
              );
            }
            if (evidenceMap.containsKey(BotMakerFeatures.evidenceKeys)) {
              caseUpdateToInfo.put(BotMakerFeatures.evidenceKeys,
                  evidenceMap.get(BotMakerFeatures.evidenceKeys));
              evidenceInfoMap.put(
                  BotMakerFeatures.evidenceKeys,
                  evidenceMap.get(BotMakerFeatures.evidenceKeys)
              );
            }
          }
        }
      }

      builder.putCollection(OPTIONAL, BotMakerFeatures.caseUpdates, caseUpdateList, Map.class);
      builder.putCollection(OPTIONAL, BotMakerFeatures.evidences,
          Lists.newArrayList(evidenceKeyToEvidenceInfo.values()), Map.class);
    }
  }

  private boolean isNewCase(CaseUpdate caseUpdate) {
    return caseUpdate.isSet(CaseUpdate._Fields.NEW_CASE) && caseUpdate.getNewCase() != null;
  }

  private boolean isNewEvidence(CaseUpdate caseUpdate) {
    return caseUpdate.isSet(CaseUpdate._Fields.NEW_EVIDENCE) && caseUpdate.getNewEvidence() != null;
  }

  private boolean isNewReport(CaseUpdate caseUpdate) {
    return caseUpdate.isSet(CaseUpdate._Fields.NEW_REPORT) && caseUpdate.getNewReport() != null;
  }

  private boolean isEvidenceRankUpdate(CaseUpdate caseUpdate) {
    return caseUpdate.isSet(CaseUpdate._Fields.EVIDENCE_RANK_UPDATE)
        && caseUpdate.getEvidenceRankUpdate() != null;
  }

  private boolean isCloseCaseUpdate(CaseUpdate caseUpdate) {
    return caseUpdate.isSet(CaseUpdate._Fields.CLOSE_CASE_UPDATE)
        && caseUpdate.getCloseCaseUpdate() != null;
  }

  private boolean isAutoCloseCaseUpdate(CaseUpdate caseUpdate) {
    return caseUpdate.isSet(CaseUpdate._Fields.AUTO_CLOSE_CASE_UPDATE)
        && caseUpdate.getAutoCloseCaseUpdate() != null;
  }

  private List<Map<String, String>> processReportFields(CaseUpdate caseUpdate) throws Exception {
    ReportContext reportContext = () -> {
      if (isNewCase(caseUpdate)) {
        return caseUpdate.getNewCase().getReport();
      } else if (isNewEvidence(caseUpdate)) {
        return caseUpdate.getNewEvidence().getReport();
      } else if (isNewReport(caseUpdate)) {
        return caseUpdate.getNewReport().getReport();
      } else {
        return null;
      }
    };

    Report report = reportContext.getReport();

    List<Map<String, String>> reportInfo = Lists.newArrayList();
    Map<String, String> reportFields = Maps.newHashMap();
    if (report != null && report.isSetReporterInfo() && report.getReporterInfo() != null
        && report.isSetReportedUserInfo() && report.getReportedUserInfo() != null) {
      if (report.getReporterInfo().isSetReporterType()
          && report.getReporterInfo().getReporterType() != null) {
        reportFields.put(BotMakerFeatures.reporterType,
            report.getReporterInfo().getReporterType().name());
      }

      if (report.getReporterInfo().isSetReporterId()) {
        reportFields.put(BotMakerFeatures.reporterId,
            String.valueOf(report.getReporterInfo().getReporterId()));
      }

      if (report.getReportedUserInfo().isSetUserId()) {
        reportFields.put(BotMakerFeatures.reportedUserId,
            String.valueOf(report.getReportedUserInfo().getUserId()));
      }
      reportInfo.add(reportFields);
    }
    return reportInfo;
  }


  private List<Map<String, String>> processEvidenceFields(CaseUpdate caseUpdate) throws Exception {
    EvidencesContext evidencesContext = () -> {
      if (isNewCase(caseUpdate)) {
        return Lists.newArrayList(caseUpdate.getNewCase().getEvidence());
      } else if (isNewEvidence(caseUpdate)) {
        return Lists.newArrayList(caseUpdate.getNewEvidence().getEvidence());
      } else if (isNewReport(caseUpdate)) {
        return Lists.newArrayList(caseUpdate.getNewReport().getEvidence());
      } else if (isCloseCaseUpdate(caseUpdate)) {
        return caseUpdate.getCloseCaseUpdate().getEvidences();
      } else if (isAutoCloseCaseUpdate(caseUpdate)) {
        return caseUpdate.getAutoCloseCaseUpdate().getEvidences();
      } else if (isEvidenceRankUpdate(caseUpdate)) {
        return Lists.newArrayList(caseUpdate.getEvidenceRankUpdate().getEvidence());
      } else {
        return Lists.newArrayList();
      }
    };

    List<Evidence> evidences = evidencesContext.getEvidences();
    List<Map<String, String>> evidenceList = Lists.newArrayList();
    if (evidences != null && evidences.size() > 0) {
      evidenceList = evidences
          .stream()
          .filter(e ->
              e.isSetEntity()
                  && e.isSetEvidenceKey()
          )
          .map(e -> {
            Map<String, String> m = Maps.newHashMap();
            m.put(BotMakerFeatures.evidenceKeys, e.getEvidenceKey());
            if (e.isSetActions()) {
              m.put(BotMakerFeatures.evidenceActions, getEvidenceActions(e));
            }
            if (e.isSetActions() && hasPolicyTags(e)) {
              m.put(BotMakerFeatures.evidenceActionPolicyTags, getEvidenceActionPolicyTags(e));
            }

            if (isTweetEntity(e)) {
              m.putAll(processTweetEvidence(e));
            } else if (isProfileEntity(e)) {
              m.putAll(processProfileEvidence(e));
            }
            if (hasEvidenceRank(e)) {
              m.putAll(processEvidenceRank(e));
            }
            return m;
          })
          .collect(Collectors.toList());
    }
    return evidenceList;
  }

  private boolean hasEvidenceRank(Evidence evidence) {
    return evidence.isSetEvidenceRank()
        && evidence.getEvidenceRank() != null;
  }

  private boolean isTweetEntity(Evidence evidence) {
    return evidence.getEntity().isSet(Entity._Fields.TWEET)
        && evidence.getEntity().getTweet() != null
        && evidence.getEntity().getTweet().isSetTweetId();
  }

  private boolean isProfileEntity(Evidence evidence) {
    return evidence.getEntity().isSet(Entity._Fields.PROFILE)
        && evidence.getEntity().getProfile() != null
        && evidence.getEntity().getProfile().isSetUserId();
  }

  private String getEvidenceActions(Evidence evidence) {
    List<String> actionNames = evidence.getActions()
        .stream()
        .map(a -> a.actionType.name())
        .collect(Collectors.toList());
    return String.join(",", actionNames);
  }

  private String getPolicyTagNames(Policy policy) {
    if (policy != null && policy.isSet()
        && policy.isSet(Policy._Fields.ATT_POLICY)
        && policy.getAttPolicy().isSetTag()) {
      return policy.getAttPolicy().getTag().name();
    } else if (policy != null && policy.isSet()
        && policy.isSet(Policy._Fields.STT_POLICY)
        && policy.getSttPolicy().isSetTags()) {
      List<String> tagNames =
          policy.getSttPolicy().getTags().stream()
              .map(tag -> tag.name()).collect(Collectors.toList());
      return String.join(",", tagNames);
    }
    return null;
  }

  private boolean hasPolicyTags(Evidence evidence) {
    return evidence.getActions()
        .stream()
        .anyMatch(a -> a.policy != null && getPolicyTagNames(a.policy) != null);
  }

  private String getEvidenceActionPolicyTags(Evidence evidence) {
      List<String> policyTagNames = evidence.getActions()
          .stream()
          .map(a -> getPolicyTagNames(a.policy))
          .filter(tag -> tag != null)
          .collect(Collectors.toList());
      return String.join(",", policyTagNames);
  }

  private Map<String, String> processTweetEvidence(Evidence evidence) {
    Map<String, String> m = new HashMap<>();
    m.put(BotMakerFeatures.evidenceEntity, Entity._Fields.TWEET.getFieldName());
    m.put(BotMakerFeatures.tweetId, String.valueOf(evidence.getEntity().getTweet().getTweetId()));

    if (evidence.getEntity().getTweet().isSetAuthorUserId()) {
      m.put(BotMakerFeatures.userId,
          String.valueOf(evidence.getEntity().getTweet().getAuthorUserId()));
    }
    return m;
  }

  private Map<String, String> processProfileEvidence(Evidence evidence) {
    Map<String, String> m = new HashMap<>();
    m.put(BotMakerFeatures.evidenceEntity, Entity._Fields.PROFILE.getFieldName());
    m.put(BotMakerFeatures.userId, String.valueOf(evidence.getEntity().getProfile().getUserId()));
    return m;
  }

  private Map<String, String> processEvidenceRank(Evidence e) {
    Map<String, String> m = new HashMap<>();
    EvidenceRank evidenceRank = e.getEvidenceRank();
    m.put(BotMakerFeatures.modelType, evidenceRank.getModelType().name());
    m.put(BotMakerFeatures.modelVersion, evidenceRank.getModelVersion().name());
    m.put(BotMakerFeatures.rank, String.valueOf(evidenceRank.getRank()));
    if (evidenceRank.isSetScore()) {
      m.put(BotMakerFeatures.score, String.valueOf(evidenceRank.getScore()));
    }
    if (evidenceRank.isSetUpdatedTimestampInMilliseconds()) {
      m.put(
          BotMakerFeatures.updatedTimestampMs,
          String.valueOf(evidenceRank.getUpdatedTimestampInMilliseconds())
      );
    }
    return m;
  }
}
