package com.twitter.scarecrow.features;

import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import com.google.common.collect.Lists;
import com.google.common.collect.Maps;

import com.twitter.relevance.feature_store.thrift.FeatureValue;
import com.twitter.reportflow.thriftjava.AdditionalReportedEntity;
import com.twitter.reportflow.thriftjava.ClientContext;
import com.twitter.reportflow.thriftjava.InAppReport;
import com.twitter.reportflow.thriftjava.ReportType;
import com.twitter.reportflow.thriftjava.ReportedEntityId;
import com.twitter.reportflow.thriftjava.VictimType;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureExtractionException;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;
import static com.twitter.botmaker.FeatureModifier.OPTIONAL;
import static com.twitter.botmaker.FeatureModifier.REQUIRED;

public class FeaturesOfInAppReport extends FeatureMapExtractor {

  private final InAppReport inAppReport;

  private static final String FAKE_ACCOUNT_SPAM_REPORT_FLOW = "SpamReportFlow.FakeAccount";
  private static final String PHISHING_SECURITY_SPAM_REPORT_FLOW =
      "SpamReportFlow.PhishingSecurity";
  private static final String HASHTAG_SPAM_REPORT_FLOW = "SpamReportFlow.Hashtag";
  private static final String ENGAGEMENT_SPAM_REPORT_FLOW = "SpamReportFlow.Engagement";
  private static final String CATCH_ALL_SPAM_REPORT_FLOW = "SpamReportFlow.CatchAll";

  private static final String FAKE_ACCOUNT_SPAM_TYPE = "FakeAccount";
  private static final String PHISHING_SECURITY = "PhishingSecurity";
  private static final String HASHTAG_SPAM_TYPE = "Hashtag";
  private static final String ENGAGEMENT_SPAM_TYPE = "Engagement";
  private static final String CATCHALL_SPAM_TYPE = "CatchAll";

  private static final String ME = "Me";
  private static final String COMPANY = "Company";
  private static final String GROUP = "Group";
  private static final String REPORTED_USER_V2 = "ReportedUser";
  private static final String SOMEONE_ELSE_V2 = "SomeoneElse";
  private static final String SELF_REPRESENT = "SelfRepresent";

  private static final String ENCOURAGING_SELF_HARM_REPORT_FLOW =
      "EncouragingSelfHarmReportFlow";
  private static final String ENCOURAGING_OR_CONTEMPLATING_SELF_HARM_REPORT_FLOW_SOMEONE_ELSE =
      "EncouragingOrContemplatingSelfHarmReportFlow.TargetingSomeoneElse";
  private static final String ENCOURAGING_OR_CONTEMPLATING_SELF_HARM_REPORT_FLOW_REPORTED_USER =
      "EncouragingOrContemplatingSelfHarmReportFlow.TargetingReportedUser";
  private static final String HARASSMENT_REPORT_FLOW = "HarassmentReportFlow";
  private static final String HATEFUL_CONDUCT_REPORT_FLOW = "HatefulConductReportFlow";
  private static final String OFFENSIVE_REPORT_FLOW = "OffensiveReportFlow";
  private static final String PRIVATE_INFO_REPORT_FLOW = "PrivateInfoReportFlow";
  private static final String SELF_HARM_REPORT_FLOW = "SelfHarmReportFlow";
  private static final String SENSITIVE_PERISCOPE_SELF_HARM_REPORT_FLOW =
      "SensitivePeriscopeReportFlow.SelfHarm";
  private static final String VIOLENCE_WITH_PPI_REPORT_FLOW =
      "ViolenceWithPPI";
  private static final String VIOLENCE_WITHOUT_PPI_REPORT_FLOW =
      "ViolenceWithoutPPI";
  private static final String SENSITIVE_MEDIA_PRIVATE_INFO_REPORT_FLOW =
      "SensitiveMediaReportFlow.IncludesPrivateInformation";
  private static final String SENSITIVE_MEDIA_DEPICTS_ME_REPORT_FLOW =
      "SensitiveMediaReportFlow.ItDepictsMe";

  private static final String CHILD_EXPLOITATION_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowConfirmedEmail.ChildExploitation";
  private static final String PRIVACY_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowConfirmedEmail.Privacy";
  private static final String VIOLENCE_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowConfirmedEmail.Violence";
  private static final String TERRORISM_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowConfirmedEmail.Terrorism";
  private static final String DEFAMATION_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowConfirmedEmail.Defamation";
  private static final String FORGERY_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowConfirmedEmail.Forgery";
  private static final String HATE_SPEECH_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowConfirmedEmail.HateSpeech";
  private static final String CHILD_EXPLOITATION_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowNoConfirmedEmail.ChildExploitation";
  private static final String PRIVACY_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowNoConfirmedEmail.Privacy";
  private static final String VIOLENCE_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowNoConfirmedEmail.Violence";
  private static final String TERRORISM_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowNoConfirmedEmail.Terrorism";
  private static final String DEFAMATION_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowNoConfirmedEmail.Defamation";
  private static final String FORGERY_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowNoConfirmedEmail.Forgery";
  private static final String HATE_SPEECH_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW =
      "AbuseNetzDGReportFlowNoConfirmedEmail.HateSpeech";

  private static final String ENCOURAGING_SELF_HARM = "encouraging_self_harm";
  private static final String HATEFUL_CONDUCT = "hateful_conduct";
  private static final String DISRESPECTFUL_OR_OFFENSIVE = "disrespectful_or_offensive";
  private static final String HARASSED = "harassed";
  private static final String PRIVATE_INFO = "private_info";
  private static final String VIOLENCE = "violence";
  private static final String SELF_HARM = "self_harm";
  private static final String RIGHT_TO_PRIVACY = "right_to_privacy";
  private static final String ABUSE_TO_DE_ILLEGAL = "abuse_to_de_illegal";

  private static final String REPORT_ADS = "report_ads";
  private static final String SENSITIVE_INFO = "sensitive_info";
  private static final String VOTING_MISINFORMATION = "voting_misinformation";

  public FeaturesOfInAppReport(InAppReport inAppReportEvent) {
    this.inAppReport = inAppReportEvent;
  }

  public String getSpamType(String reportFlowName) {
    if (reportFlowName == null || reportFlowName.isEmpty()) {
      return "";
    }
    switch(reportFlowName) {
      case FAKE_ACCOUNT_SPAM_REPORT_FLOW: return FAKE_ACCOUNT_SPAM_TYPE;
      case PHISHING_SECURITY_SPAM_REPORT_FLOW: return PHISHING_SECURITY;
      case ENGAGEMENT_SPAM_REPORT_FLOW: return ENGAGEMENT_SPAM_TYPE;
      case HASHTAG_SPAM_REPORT_FLOW: return HASHTAG_SPAM_TYPE;
      case CATCH_ALL_SPAM_REPORT_FLOW: return CATCHALL_SPAM_TYPE;
      default: return "";
    }
  }

  public String getAbuseType(String reportFlowName) {
    if (reportFlowName == null || reportFlowName.isEmpty()) {
      return "";
    }
    switch (reportFlowName) {
      case HARASSMENT_REPORT_FLOW: return HARASSED;
      case HATEFUL_CONDUCT_REPORT_FLOW: return HATEFUL_CONDUCT;
      case OFFENSIVE_REPORT_FLOW: return DISRESPECTFUL_OR_OFFENSIVE;
      case PRIVATE_INFO_REPORT_FLOW:
      case SENSITIVE_MEDIA_PRIVATE_INFO_REPORT_FLOW:
      case VIOLENCE_WITH_PPI_REPORT_FLOW: return PRIVATE_INFO;
      case VIOLENCE_WITHOUT_PPI_REPORT_FLOW: return VIOLENCE;
      case SELF_HARM_REPORT_FLOW:
      case SENSITIVE_PERISCOPE_SELF_HARM_REPORT_FLOW:
      case ENCOURAGING_OR_CONTEMPLATING_SELF_HARM_REPORT_FLOW_SOMEONE_ELSE:
      case ENCOURAGING_OR_CONTEMPLATING_SELF_HARM_REPORT_FLOW_REPORTED_USER: return SELF_HARM;
      case ENCOURAGING_SELF_HARM_REPORT_FLOW: return ENCOURAGING_SELF_HARM;
      case SENSITIVE_MEDIA_DEPICTS_ME_REPORT_FLOW: return RIGHT_TO_PRIVACY;
      case CHILD_EXPLOITATION_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case PRIVACY_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case VIOLENCE_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case TERRORISM_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case DEFAMATION_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case FORGERY_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case HATE_SPEECH_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case CHILD_EXPLOITATION_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case PRIVACY_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case VIOLENCE_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case TERRORISM_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case DEFAMATION_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case FORGERY_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
      case HATE_SPEECH_NO_CONFIRMED_EMAIL_ABUSE_REPORT_FLOW:
        return ABUSE_TO_DE_ILLEGAL;
      default: return "";
    }
  }

  public String getVictimTypeV2(VictimType victimType) {
    if (victimType == null) {
      return "";
    }

    switch(victimType) {
      case REPORTING_USER: return ME;
      case REPORTED_USER: return REPORTED_USER_V2;
      case COMPANY_OF_REPORTING_USER: return COMPANY;
      case REPRESENTATION_OF_REPORTING_USER: return SELF_REPRESENT;
      case SOMEONE_ELSE: return SOMEONE_ELSE_V2;
      case GROUP: return GROUP;
      default: return "";
    }
  }

  public String getReportType(ReportType reportType) {
    if (reportType == null) {
      return "";
    }

    switch (reportType) {
      case ADS: return REPORT_ADS;
      case SENSITIVE_MEDIA: return SENSITIVE_INFO;
      case MISINFORMATION: return VOTING_MISINFORMATION;
      default: return reportType.toString().toLowerCase();
    }
  }

   public void processAdditionalReportedDMs(
      FeatureMapBuilder builder)
      throws FeatureExtractionException {
    List<Long> additionalReportedDMList = Lists.newArrayList();
    for (AdditionalReportedEntity additionalReportedEntity
        : inAppReport.getAdditionalReportedEntities()) {
      if (additionalReportedEntity.reportedEntityId.getSetField()
          == ReportedEntityId._Fields.DM_ID) {
        additionalReportedDMList.add(additionalReportedEntity.reportedEntityId.getDmId());
      }
    }

    if (!additionalReportedDMList.isEmpty()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.additionalReportedDMs,
          FeatureValue.longListValue(additionalReportedDMList));
    }
  }

  public void processAdditionalReportedTweets(
      FeatureMapBuilder builder)
      throws FeatureExtractionException {

    List<Map<String, String>> tweetList = Lists.newArrayList();
    for (AdditionalReportedEntity additionalReportedEntity
        : inAppReport.getAdditionalReportedEntities()) {
      Map<String, String> additionalTweet = Maps.newHashMap();
      additionalTweet.put(BotMakerFeatures.tweetId,
          String.valueOf(additionalReportedEntity.reportedEntityId.getTweetId()));
      tweetList.add(additionalTweet);
    }
    builder.putValue(OPTIONAL, BotMakerFeatures.associatedTweets,
        FeatureValue.strMapListValue(tweetList));
    builder.putValue(OPTIONAL, BotMakerFeatures.associatedTweetsV2,
        FeatureValue.strMapListValue(tweetList));
  }

  private void processMisinformationSelections(FeatureMapBuilder builder)
      throws FeatureExtractionException {
    List<String> selections = inAppReport.completedSteps.stream()
        .flatMap(s -> {
          switch (s.getSetField()) {
            case SINGLE_SELECTION:
              return Stream.of(s.getSingleSelection().option);
            case MULTI_SELECTION:
              return s.getMultiSelection().options.stream();
            default:
              return Stream.of();
          }
        })
        .filter(option -> option.startsWith("GeneralMisinformation"))
        .collect(Collectors.toList());

    builder.putValue(OPTIONAL,
        BotMakerFeatures.misinformationSelections, FeatureValue.strListValue(selections));
  }

  private void buildClientContextInputFeatures(FeatureMapBuilder builder,
                                               ClientContext clientContext)
      throws FeatureExtractionException {
      if (clientContext.isSetApplicationId()) {
        builder.putValue(OPTIONAL,
            BotMakerFeatures.applicationId, clientContext.getApplicationId());
      }

      if (clientContext.isSetLocation()) {
        builder.putValue(OPTIONAL,
            BotMakerFeatures.clientLocation, clientContext.getLocation());
      }

      if (clientContext.isSetReferrer()) {
        builder.putValue(OPTIONAL,
            BotMakerFeatures.clientReferrer, clientContext.getReferrer());
      }

      if (clientContext.isSetUserAgent()) {
        builder.putValue(OPTIONAL,
            BotMakerFeatures.userAgent, clientContext.getUserAgent());
      }

      if (clientContext.isSetMetadata()) {
        for (Map.Entry<String, String> entry
            : clientContext.getMetadata().entrySet()) {
          builder.putValue(OPTIONAL, entry.getKey(), entry.getValue());
        }
      }

      if (clientContext.isSetIpAddress()) {
        builder.putValue(OPTIONAL, BotMakerFeatures.auditIp, clientContext.getIpAddress());
      }
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {

    String spamType = inAppReport.getReportType() == ReportType.SPAM
        ? getSpamType(inAppReport.getReportFlowName()) : "";
    String abuseType = inAppReport.getReportType() == ReportType.ABUSE
        || inAppReport.getReportType() == ReportType.GERMANY_ILLEGAL
        ? getAbuseType(inAppReport.getReportFlowName()) : "";
    String victimTypeV2 = inAppReport.isSetVictimType()
        ? getVictimTypeV2(inAppReport.getVictimType()) : "";

    builder
        .putValue(REQUIRED, BotMakerFeatures.reporterId,
            inAppReport.getReporterId())
        .putValue(REQUIRED, BotMakerFeatures.reportedAs,
            getReportType(inAppReport.getReportType()));

    if (inAppReport.isSetReportedUserId()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.reportedUserId,
          inAppReport.getReportedUserId());
    }

    if (!spamType.isEmpty()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.spamType, spamType);
    }

    if (!abuseType.isEmpty()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.abuseType, abuseType);
    }

    if (!victimTypeV2.isEmpty()) {
      builder.putValue(OPTIONAL, BotMakerFeatures.victimTypeV2, victimTypeV2);
    }

    if (inAppReport.getReportType() == ReportType.MISINFORMATION) {
      processMisinformationSelections(builder);
    }

    if (inAppReport.isSetClientContext()) {
      buildClientContextInputFeatures(builder, inAppReport.clientContext);
    }
  }
}
