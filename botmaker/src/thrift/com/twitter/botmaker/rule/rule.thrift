namespace java com.twitter.botmaker.rule.thriftjava
namespace py gen.twitter.botmaker.rule.thriftpy

typedef i64 DateTimeMilliSeconds

struct BotMakerRule {
  1:  required i64 id
  2:  required string name
  3:  required string description = ""
  5:  required string eventType
  6:  required string booleanExpression
  7:  required string actionExpression

  8:  optional string lastUpdatedBy

  9:  optional DateTimeMilliSeconds createdAt
  10: optional DateTimeMilliSeconds expiryDate
  11: optional DateTimeMilliSeconds lastUpdatedAt

  12: optional string creator

  13: optional string team = ""

  14: optional map<string, list<ActionLimit>> actionLimits = {}

}(persisted='true')

struct ActionLimit {
  1: required i64 quota
  2: required i32 periodInMinutes
  3: required RateLimitingStrategy strategy
  4: required i32 version
}(persisted='true')

enum RateLimitingStrategy {
  TOKEN_BUCKET = 0,
  HALT_UNTIL_HUMAN_INTERVENTION = 1,
}

struct BotMakerDerivedFeature {
  1: required string name
  2: required string description = ""
  3: required string featureExpression
  4: optional bool logged
  5: optional DateTimeMilliSeconds createdAt
  6: optional DateTimeMilliSeconds lastUpdatedAt
  7: optional string returnType = "Auto"
}(persisted='true')

struct BotMakerRulesPackage {
  1: required list<BotMakerRule> rules
  2: required list<BotMakerDerivedFeature> derivedFeatures = []
}
