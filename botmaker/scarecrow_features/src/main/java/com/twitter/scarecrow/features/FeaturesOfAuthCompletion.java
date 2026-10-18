package com.twitter.scarecrow.features;

import java.util.HashMap;
import java.util.Map;
import java.util.stream.Collectors;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import com.twitter.botmaker.FeatureModifier;
import com.twitter.satellite.surface.authentication.thriftjava.AuthCompletion;
import com.twitter.satellite.surface.authentication.thriftjava.AuthCompletionMeta;
import com.twitter.satellite.surface.authentication.thriftjava.AuthFlowAudit;
import com.twitter.satellite.surface.authentication.thriftjava.AuthorizeApplicationFlowAudit;
import com.twitter.satellite.surface.authentication.thriftjava.AuthorizeContributorFlowAudit;
import com.twitter.satellite.surface.authentication.thriftjava.CrossLoginFlowAudit;
import com.twitter.satellite.surface.authentication.thriftjava.LiteLoginAuthenticateFlowAudit;
import com.twitter.satellite.surface.authentication.thriftjava.LiteLoginFlowAudit;
import com.twitter.satellite.surface.authentication.thriftjava.LoginFlowAudit;
import com.twitter.satellite.surface.authentication.thriftjava.PasswordResetFlowAudit;
import com.twitter.spam.botmaker_features.BotMakerFeatures;
import com.twitter.spam.botmaker_features.FeatureMapBuilder;
import com.twitter.spam.botmaker_features.FeatureMapExtractor;

public class FeaturesOfAuthCompletion extends FeatureMapExtractor {
  private static final Logger LOG = LoggerFactory.getLogger(FeaturesOfAuthCompletion.class);

  private final AuthCompletion authCompletion;

  public FeaturesOfAuthCompletion(AuthCompletion authCompletion) {
    this.authCompletion = authCompletion;
  }

  @Override
  public void apply(FeatureMapBuilder builder) throws Exception {
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.token, authCompletion.getToken());
    putMetaValues(builder, authCompletion.getMeta());
    builder.putValue(FeatureModifier.REQUIRED, BotMakerFeatures.completedFlow,
        toStrMap(authCompletion.getCompletedFlow()));
    if (authCompletion.isSetConnectedFlows()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.connectedFlows,
          authCompletion.getConnectedFlows().stream().map(this::toStrMap)
              .collect(Collectors.toList()));

    }
    if (authCompletion.isSetAbandonedFlows()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.abandonedFlows,
          authCompletion.getAbandonedFlows().stream().map(this::toStrMap)
              .collect(Collectors.toList()));
    }
  }

  private void putMetaValues(FeatureMapBuilder builder, AuthCompletionMeta meta) throws Exception {
    if (meta.isSetIsLimitedTimeline()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isLimitedTimeline,
          meta.isIsLimitedTimeline());
    }
    if (meta.isSetIsTimelineSupportedClient()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.isTimelineSupportedClient,
          meta.isIsTimelineSupportedClient());
    }
    if (meta.isSetTransactionTimes()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.transactionTimes,
          meta.getTransactionTimes());
    }
    if (meta.isSetReferralSequence()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.referralSequence,
          meta.getReferralSequence());
    }
    if (meta.isSetOcfFlowTokenSequence()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.ocfFlowTokenSequence,
          meta.getOcfFlowTokenSequence());
    }
    if (meta.isSetClientApplicationIdSequence()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.clientApplicationIdSequence,
          meta.getClientApplicationIdSequence());
    }
    if (meta.isSetKnownDeviceTokenSequence()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.knownDeviceTokenSequence,
          meta.getKnownDeviceTokenSequence());
    }
    if (meta.isSetSessionHashSequence()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.sessionHashSequence,
          meta.getSessionHashSequence());
    }
    if (meta.isSetIpAddressSequence()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.ipAddressSequence,
          meta.getIpAddressSequence());
    }
    if (meta.isSetLocationCodeSequence()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.locationCodeSequence,
          meta.getLocationCodeSequence());
    }
    if (meta.isSetUserAgentSequence()) {
      builder.putValue(FeatureModifier.OPTIONAL, BotMakerFeatures.userAgentSequence,
          meta.getUserAgentSequence());
    }
  }

  private Map<String, String> toStrMap(AuthFlowAudit completedFlow) {
    AuthFlowAudit._Fields setField = completedFlow.getSetField();
    Map<String, String> flowMap = new HashMap<>();
    flowMap.put(BotMakerFeatures.flowType, setField.getFieldName());
    switch (setField) {
      case LOGIN:
        putLoginToMap(flowMap, completedFlow.getLogin());
        break;
      case PASSWORD_RESET:
        putPasswordResetToMap(flowMap, completedFlow.getPasswordReset());
        break;
      case AUTHORIZE_APPLICATION:
        putAuthorizeApplicationToMap(flowMap, completedFlow.getAuthorizeApplication());
        break;
      case AUTHORIZE_CONTRIBUTOR:
        putAuthorizeContributorToMap(flowMap, completedFlow.getAuthorizeContributor());
        break;
      case LITE_LOGIN:
        putLiteLoginToMap(flowMap, completedFlow.getLiteLogin());
        break;
      case LITE_LOGIN_AUTHENTICATE:
        putLiteLoginAuthenticateToMap(flowMap, completedFlow.getLiteLoginAuthenticate());
        break;
      case CROSS_LOGIN:
        putCrossLoginToMap(flowMap, completedFlow.getCrossLogin());
        break;
      default:
        LOG.warn("unknown AuthFlowAudit type: " + setField.getFieldName());
        break;
    }
    return flowMap;
  }

  private void putLoginToMap(Map<String, String> flowMap, LoginFlowAudit login) {
    flowMap.put("flowState", login.getFlowState());
    flowMap.put("flowSystem", login.getFlowSystem());
    if (login.isSetUserId()) {
      flowMap.put("userId", String.valueOf(login.getUserId()));
    }
    if (login.isSetGrantedAccessTokenId()) {
      flowMap.put("grantedAccessTokenId", String.valueOf(login.getGrantedAccessTokenId()));
    }
    if (login.isSetGrantedSessionHash()) {
      flowMap.put("grantedSessionHash", login.getGrantedSessionHash());
    }
    if (login.isSetFailures()) {
      flowMap.put("failures", String.join(",", login.getFailures()));
    }
    if (login.isSetIdentificationAttempted()) {
      flowMap.put("identificationAttempted", String.valueOf(login.isIdentificationAttempted()));
    }
    if (login.isSetIdentificationPassed()) {
      flowMap.put("identificationPassed", String.valueOf(login.isIdentificationPassed()));
    }
    if (login.isSetPassedAccountIdentifier()) {
      flowMap.put("passedAccountIdentifier", login.getPassedAccountIdentifier());
    }
    if (login.isSetPassedAccountIdentifierRaw()) {
      flowMap.put("passedAccountIdentifierRaw", login.getPassedAccountIdentifierRaw());
    }
    if (login.isSetSelectionDenied()) {
      flowMap.put("selectionDenied", String.valueOf(login.isSelectionDenied()));
    }
    if (login.isSetSelectionRequired()) {
      flowMap.put("selectionRequired", String.valueOf(login.isSelectionRequired()));
    }
    if (login.isSetSelectionSkipped()) {
      flowMap.put("selectionSkipped", String.valueOf(login.isSelectionSkipped()));
    }
    if (login.isSetSelectionAttempted()) {
      flowMap.put("selectionAttempted", String.valueOf(login.isSelectionAttempted()));
    }
    if (login.isSetSelectionPassed()) {
      flowMap.put("selectionPassed", String.valueOf(login.isSelectionPassed()));
    }
    if (login.isSetPassedSelectionIdentifier()) {
      flowMap.put("passedSelectionIdentifier", login.getPassedSelectionIdentifier());
    }
    if (login.isSetRecaptchaRequired()) {
      flowMap.put("recaptchaRequired", String.valueOf(login.isRecaptchaRequired()));
    }
    if (login.isSetRecaptchaSkipped()) {
      flowMap.put("recaptchaSkipped", String.valueOf(login.isRecaptchaSkipped()));
    }
    if (login.isSetRecaptchaAttempted()) {
      flowMap.put("recaptchaAttempted", String.valueOf(login.isRecaptchaAttempted()));
    }
    if (login.isSetRecaptchaPassed()) {
      flowMap.put("recaptchaPassed", String.valueOf(login.isRecaptchaPassed()));
    }
    if (login.isSetSolveAccountKnowledgeChallengeRequired()) {
      flowMap.put("solveAccountKnowledgeChallengeRequired",
          String.valueOf(login.isSolveAccountKnowledgeChallengeRequired()));
    }
    if (login.isSetSolveAccountKnowledgeChallengeSkipped()) {
      flowMap.put("solveAccountKnowledgeChallengeSkipped",
          String.valueOf(login.isSolveAccountKnowledgeChallengeSkipped()));
    }
    if (login.isSetSolveAccountKnowledgeChallengeAttempted()) {
      flowMap.put("solveAccountKnowledgeChallengeAttempted",
          String.valueOf(login.isSolveAccountKnowledgeChallengeAttempted()));
    }
    if (login.isSetSolveAccountKnowledgeChallengePassed()) {
      flowMap.put("solveAccountKnowledgeChallengePassed",
          String.valueOf(login.isSolveAccountKnowledgeChallengePassed()));
    }
    if (login.isSetAccountKnowledgeChallenge()) {
      flowMap.put("accountKnowledgeChallenge", login.getAccountKnowledgeChallenge());
    }
    if (login.isSetVerifyTemporaryPasswordRequired()) {
      flowMap.put("verifyTemporaryPasswordRequired",
          String.valueOf(login.isVerifyTemporaryPasswordRequired()));
    }
    if (login.isSetVerifyTemporaryPasswordAttempted()) {
      flowMap.put("verifyTemporaryPasswordAttempted",
          String.valueOf(login.isVerifyTemporaryPasswordAttempted()));
    }
    if (login.isSetVerifyTemporaryPasswordPassed()) {
      flowMap.put("verifyTemporaryPasswordPassed",
          String.valueOf(login.isVerifyTemporaryPasswordPassed()));
    }
    if (login.isSetVerifyPasswordAttempted()) {
      flowMap.put("verifyPasswordAttempted", String.valueOf(login.isVerifyPasswordAttempted()));
    }
    if (login.isSetVerifyPasswordPassed()) {
      flowMap.put("verifyPasswordPassed", String.valueOf(login.isVerifyPasswordPassed()));
    }
    if (login.isSetVerifyPasswordByRecall()) {
      flowMap.put("verifyPasswordByRecall", String.valueOf(login.isVerifyPasswordByRecall()));
    }
    if (login.isSetVerifyPasswordByReset()) {
      flowMap.put("verifyPasswordByReset", String.valueOf(login.isVerifyPasswordByReset()));
    }
    if (login.isSetVerifyAppPasswordRequired()) {
      flowMap.put("verifyAppPasswordRequired", String.valueOf(login.isVerifyAppPasswordRequired()));
    }
    if (login.isSetVerifyAppPasswordAttempted()) {
      flowMap.put("verifyAppPasswordAttempted",
          String.valueOf(login.isVerifyAppPasswordAttempted()));
    }
    if (login.isSetVerifyAppPasswordPassed()) {
      flowMap.put("verifyAppPasswordPassed", String.valueOf(login.isVerifyAppPasswordPassed()));
    }
    if (login.isSetVerifySecondFactorRequired()) {
      flowMap.put("verifySecondFactorRequired",
          String.valueOf(login.isVerifySecondFactorRequired()));
    }
    if (login.isSetVerifySecondFactorSwitched()) {
      flowMap.put("verifySecondFactorSwitched",
          String.valueOf(login.isVerifySecondFactorSwitched()));
    }
    if (login.isSetVerifySecondFactorAttempted()) {
      flowMap.put("verifySecondFactorAttempted",
          String.valueOf(login.isVerifySecondFactorAttempted()));
    }
    if (login.isSetVerifySecondFactorPassed()) {
      flowMap.put("verifySecondFactorPassed", String.valueOf(login.isVerifySecondFactorPassed()));
    }
    if (login.isSetPassedSecondFactor()) {
      flowMap.put("passedSecondFactor", login.getPassedSecondFactor());
    }
    if (login.isSetReactivateAccountRequired()) {
      flowMap.put("reactivateAccountRequired", String.valueOf(login.isReactivateAccountRequired()));
    }
    if (login.isSetReactivateAccountPassed()) {
      flowMap.put("reactivateAccountPassed", String.valueOf(login.isReactivateAccountPassed()));
    }
    if (login.isSetAuthenticateAccountSkipped()) {
      flowMap.put("authenticateAccountSkipped",
          String.valueOf(login.isAuthenticateAccountSkipped()));
    }
    if (login.isSetAuthenticateAccountPassed()) {
      flowMap.put("authenticateAccountPassed", String.valueOf(login.isAuthenticateAccountPassed()));
    }
    if (login.isSetFinalRuleIdForAccountSelection()) {
      flowMap.put("finalRuleIdForAccountSelection",
          String.valueOf(login.getFinalRuleIdForAccountSelection()));
    }
    if (login.isSetFinalRuleIdForRecaptcha()) {
      flowMap.put("finalRuleIdForRecaptcha", String.valueOf(login.getFinalRuleIdForRecaptcha()));
    }
    if (login.isSetFinalRuleIdForTemporaryPassword()) {
      flowMap.put("finalRuleIdForTemporaryPassword",
          String.valueOf(login.getFinalRuleIdForTemporaryPassword()));
    }
    if (login.isSetFinalRuleIdForAccountKnowledgeChallenge()) {
      flowMap.put("finalRuleIdForAccountKnowledgeChallenge",
          String.valueOf(login.getFinalRuleIdForAccountKnowledgeChallenge()));
    }
    if (login.isSetFinalRuleIdForLoginFlowFailure()) {
      flowMap.put("finalRuleIdForLoginFlowFailure",
          String.valueOf(login.getFinalRuleIdForLoginFlowFailure()));
    }
    if (login.isSetRuleIdsForAccountSelection()) {
      flowMap.put("ruleIdsForAccountSelection",
          login.getRuleIdsForAccountSelection().stream().map(String::valueOf)
              .collect(Collectors.joining(",")));
    }
  }

  private void putPasswordResetToMap(
      Map<String, String> flowMap,
      PasswordResetFlowAudit passwordReset
  ) {
    flowMap.put("flowState", passwordReset.getFlowState());
    flowMap.put("flowSystem", passwordReset.getFlowSystem());
    if (passwordReset.isSetUserId()) {
      flowMap.put("userId", String.valueOf(passwordReset.getUserId()));
    }
    if (passwordReset.isSetGrantedAccessTokenId()) {
      flowMap.put("grantedAccessTokenId", String.valueOf(passwordReset.getGrantedAccessTokenId()));
    }
    if (passwordReset.isSetGrantedSessionHash()) {
      flowMap.put("grantedSessionHash", passwordReset.getGrantedSessionHash());
    }
    if (passwordReset.isSetFailures()) {
      flowMap.put("failures", String.join(",", passwordReset.getFailures()));
    }
    if (passwordReset.isSetIdentificationAttempted()) {
      flowMap.put("identificationAttempted",
          String.valueOf(passwordReset.isIdentificationAttempted()));
    }
    if (passwordReset.isSetIdentificationPassed()) {
      flowMap.put("identificationPassed", String.valueOf(passwordReset.isIdentificationPassed()));
    }
    if (passwordReset.isSetPassedAccountIdentifier()) {
      flowMap.put("passedAccountIdentifier", passwordReset.getPassedAccountIdentifier());
    }
    if (passwordReset.isSetPassedAccountIdentifierRaw()) {
      flowMap.put("passedAccountIdentifierRaw", passwordReset.getPassedAccountIdentifierRaw());
    }
    if (passwordReset.isSetSelectionRequired()) {
      flowMap.put("selectionRequired", String.valueOf(passwordReset.isSelectionRequired()));
    }
    if (passwordReset.isSetSelectionSkipped()) {
      flowMap.put("selectionSkipped", String.valueOf(passwordReset.isSelectionSkipped()));
    }
    if (passwordReset.isSetSelectionBypassed()) {
      flowMap.put("selectionBypassed", String.valueOf(passwordReset.isSelectionBypassed()));
    }
    if (passwordReset.isSetSelectionAttempted()) {
      flowMap.put("selectionAttempted", String.valueOf(passwordReset.isSelectionAttempted()));
    }
    if (passwordReset.isSetSelectionPassed()) {
      flowMap.put("selectionPassed", String.valueOf(passwordReset.isSelectionPassed()));
    }
    if (passwordReset.isSetVerifyResetMethodChoices()) {
      flowMap.put("verifyResetMethodChoices",
          String.valueOf(passwordReset.getVerifyResetMethodChoices()));
    }
    if (passwordReset.isSetVerifyResetMethodAttempts()) {
      flowMap.put("verifyResetMethodAttempts",
          String.valueOf(passwordReset.getVerifyResetMethodAttempts()));
    }
    if (passwordReset.isSetVerifyResetMethodFailures()) {
      flowMap.put("verifyResetMethodFailures",
          String.valueOf(passwordReset.getVerifyResetMethodFailures()));
    }
    if (passwordReset.isSetVerifyResetMethodPassBySms()) {
      flowMap.put("verifyResetMethodPassBySms",
          String.valueOf(passwordReset.isVerifyResetMethodPassBySms()));
    }
    if (passwordReset.isSetVerifyResetMethodPassByEmail()) {
      flowMap.put("verifyResetMethodPassByEmail",
          String.valueOf(passwordReset.isVerifyResetMethodPassByEmail()));
    }
    if (passwordReset.isSetSetNewPasswordAttempted()) {
      flowMap.put("setNewPasswordAttempted",
          String.valueOf(passwordReset.isSetNewPasswordAttempted()));
    }
    if (passwordReset.isSetSetNewPasswordFailures()) {
      flowMap.put("setNewPasswordFailures",
          String.join(",", passwordReset.getSetNewPasswordFailures()));
    }
    if (passwordReset.isSetSetNewPasswordPassed()) {
      flowMap.put("setNewPasswordPassed", String.valueOf(passwordReset.isSetNewPasswordPassed()));
    }
    if (passwordReset.isSetAuthenticationSkipped()) {
      flowMap.put("authenticationSkipped", String.valueOf(passwordReset.isAuthenticationSkipped()));
    }
    if (passwordReset.isSetAuthenticationFailed()) {
      flowMap.put("authenticationFailed", String.valueOf(passwordReset.isAuthenticationFailed()));
    }
    if (passwordReset.isSetAuthenticationPassed()) {
      flowMap.put("authenticationPassed", String.valueOf(passwordReset.isAuthenticationPassed()));
    }
    if (passwordReset.isSetAuthenticationDeferred()) {
      flowMap.put("authenticationDeferred",
          String.valueOf(passwordReset.isAuthenticationDeferred()));
    }
    if (passwordReset.isSetAuthenticationDeferral()) {
      flowMap.put("authenticationDeferral", passwordReset.getAuthenticationDeferral());
    }
    if (passwordReset.isSetSubmitResetSurveyPassed()) {
      flowMap.put("submitResetSurveyPassed",
          String.valueOf(passwordReset.isSubmitResetSurveyPassed()));
    }
    if (passwordReset.isSetResetSurveyResponse()) {
      flowMap.put("resetSurveyResponse", passwordReset.getResetSurveyResponse());
    }
    if (passwordReset.isSetFinalRuleIdForAccountSelection()) {
      flowMap.put("finalRuleIdForAccountSelection",
          String.valueOf(passwordReset.getFinalRuleIdForAccountSelection()));
    }
  }

  private void putAuthorizeApplicationToMap(
      Map<String, String> flowMap,
      AuthorizeApplicationFlowAudit authorizeApplication
  ) {
    flowMap.put("flowState", authorizeApplication.getFlowState());
    flowMap.put("flowSystem", authorizeApplication.getFlowSystem());
    if (authorizeApplication.isSetUserId()) {
      flowMap.put("userId", String.valueOf(authorizeApplication.getUserId()));
    }
    if (authorizeApplication.isSetClientApplicationId()) {
      flowMap.put("clientApplicationId",
          String.valueOf(authorizeApplication.getClientApplicationId()));
    }
  }

  private void putAuthorizeContributorToMap(
      Map<String, String> flowMap,
      AuthorizeContributorFlowAudit authorizeContributor
  ) {
    flowMap.put("flowState", authorizeContributor.getFlowState());
    flowMap.put("flowSystem", authorizeContributor.getFlowSystem());
    if (authorizeContributor.isSetUserId()) {
      flowMap.put("userId", String.valueOf(authorizeContributor.getUserId()));
    }
    if (authorizeContributor.isSetContributorId()) {
      flowMap.put("contributorId", String.valueOf(authorizeContributor.getContributorId()));
    }
  }

  private void putLiteLoginToMap(
      Map<String, String> flowMap,
      LiteLoginFlowAudit liteLogin
  ) {
    flowMap.put("flowState", liteLogin.getFlowState());
    flowMap.put("flowSystem", liteLogin.getFlowSystem());
    if (liteLogin.isSetUserId()) {
      flowMap.put("userId", String.valueOf(liteLogin.getUserId()));
    }
    if (liteLogin.isSetGrantRestrictedTokenPassed()) {
      flowMap.put("grantRestrictedTokenPassed",
          String.valueOf(liteLogin.isGrantRestrictedTokenPassed()));
    }
    if (liteLogin.isSetSessionHash()) {
      flowMap.put("sessionHash", liteLogin.getSessionHash());
    }
  }

  private void putLiteLoginAuthenticateToMap(
      Map<String, String> flowMap,
      LiteLoginAuthenticateFlowAudit liteLoginAuthenticate
  ) {
    flowMap.put("flowState", liteLoginAuthenticate.getFlowState());
    flowMap.put("flowSystem", liteLoginAuthenticate.getFlowSystem());
    if (liteLoginAuthenticate.isSetUserId()) {
      flowMap.put("userId", String.valueOf(liteLoginAuthenticate.getUserId()));
    }
    if (liteLoginAuthenticate.isSetUpgradeTokenPassed()) {
      flowMap.put("upgradeTokenPassed",
          String.valueOf(liteLoginAuthenticate.isUpgradeTokenPassed()));
    }
    if (liteLoginAuthenticate.isSetSessionHash()) {
      flowMap.put("sessionHash", liteLoginAuthenticate.getSessionHash());
    }
  }

  private void putCrossLoginToMap(
      Map<String, String> flowMap,
      CrossLoginFlowAudit crossLogin
  ) {
    flowMap.put("flowState", crossLogin.getFlowState());
    flowMap.put("flowSystem", crossLogin.getFlowSystem());
    if (crossLogin.isSetUserId()) {
      flowMap.put("userId", String.valueOf(crossLogin.getUserId()));
    }
    if (crossLogin.isSetClientApplicationId()) {
      flowMap.put("clientApplicationId",
          String.valueOf(crossLogin.getClientApplicationId()));
    }
    if (crossLogin.isSetGrantedAccessTokenId()) {
      flowMap.put("grantedAccessTokenId",
          String.valueOf(crossLogin.getGrantedAccessTokenId()));
    }
    if (crossLogin.isSetGrantedSessionHash()) {
      flowMap.put("grantedSessionHash", crossLogin.getGrantedSessionHash());
    }
    if (crossLogin.isSetFromSessionHash()) {
      flowMap.put("fromSessionHash", crossLogin.getFromSessionHash());
    }
    if (crossLogin.isSetFailures()) {
      flowMap.put("failures", String.join(",", crossLogin.getFailures()));
    }
  }
}
