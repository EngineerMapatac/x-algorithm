namespace java com.twitter.botmaker.app.thriftjava

include "com/twitter/relevance/feature_store/fs_common.thrift"
include "com/twitter/botmaker/rule/rule.thrift"
include "com/twitter/botmaker/runtime/runtime.thrift"
include "com/twitter/botmaker/botmaker.thrift"
include "com/twitter/service/scarecrow/gen/tiered_actions.thrift"
include "com/twitter/safety_systems/simple_list/simple_list.thrift"

struct AppRequest {
  1: string eventType
  2: binary inputFeature
}

struct AppResponse {
  1: binary outputFeatures
}

struct RunExprRequest {
  1: required string expression
  2: optional string eventType
  3: optional string inputFeatures
}

struct RunExprResponse {
  1: required string result
  2: optional string display = "text"
}

struct RunExprWithFeatureDataRequest {
  1: required string expression
  2: optional string eventType
  3: optional map<string, fs_common.FeatureData> inputFeatures
}

struct RunExprWithFeatureDataResponse {
  1: required string result
  2: optional string display = "text"
}

struct CheckPackageRequest {
  1: required rule.BotMakerRulesPackage rulesPackage
}

struct CheckPackageResponse {
  1: required bool result
}

struct CheckSyntaxRequest {
  1: optional string eventType
  2: required string expression
  3: optional string returnType
  4: optional bool isDerivedFeature
}

struct CheckSyntaxResponse {
  1: required bool result
}

struct getBotMakerDocRequest {
  1: required string category
}

struct BotMakerDocEntry {
  1: required string name
  2: required string description
  3: required bool isDerivedFeature
  4: required list<string> argTypes
  5: required list<string> arguments
  6: required string returnType
  8: optional string actionLevelName
}

struct getBotMakerDocResponse {
  1: required list<BotMakerDocEntry> entries
}

struct getAppInfoResponse {
  1: required map<string, string> infos
}

struct getEventTypesResponse {
  1: required map<string, runtime.BotMakerEventTypeConfig> infos
}

struct JsonRequest {
  1: string eventType
  2: string inputJson
}

struct JsonResponse {
  1: string outputJson
}

struct GetTeamsResponse {
  1: required list<runtime.TeamConfig> teams
}

struct SimpleListValue {
  1: required string value
  2: required simple_list.PatternType patternType
}

struct SimpleListTestRequest {
  1: required set<SimpleListValue> simpleListValues
  2: required list<string> testInputs
}

struct SimpleListTestResponse {
  1: required list<SimpleListMatchResult> perInputMatches
}

struct SimpleListMatchResult {
  1: required list<string> matchesListExtracted
  2: required list<string> extractListExtracted
}

union AnalysisTarget {
  1: i64 ruleId
}

struct AnalysisPerTargetResponse {
  1: map<string, fs_common.FeatureData> result
}

struct AnalysisResponse {
  1: map<AnalysisTarget, AnalysisPerTargetResponse> targetResponseMap
}


service AppService {

  AppResponse botMakerAppCheckEvent(
    1: AppRequest request
  )

  tiered_actions.TieredAction botMakerCheckEvent(
    1: botmaker.BotMakerEvent event
  )

  RunExprResponse botMakerRunExpression(
    1: RunExprRequest request
  )

  RunExprWithFeatureDataResponse botMakerRunExprWithFeatureData(
    1: RunExprWithFeatureDataRequest request
  )

  CheckPackageResponse botMakerCheckPackage(
    1: CheckPackageRequest request
  )

  CheckSyntaxResponse botMakerCheckSyntax(
    1: CheckSyntaxRequest request
  )

  getBotMakerDocResponse botMakerGetBotMakerDoc(
    1: getBotMakerDocRequest request
  )

  getAppInfoResponse botMakerGetAppInfo()

  getEventTypesResponse botMakerGetEventTypes()

  JsonResponse checkJsonEvent(
    1: JsonRequest request
  )

  GetTeamsResponse getTeams()

  SimpleListTestResponse testSimpleListValues(
    1: SimpleListTestRequest request
  )
  
  AnalysisResponse analyzeCode()
  
  
}

struct AppClientConfig {
  1: string endpoint
  2: string description
  3: required string clientId
  4: required string servicePath
  10: optional i64 requestTimeoutMillis = 2000
  11: optional i64 sessionTimeoutMillis = 500
  12: optional list<i64> partitions = []
  13: optional i64 partitionFailovers = 2
}
