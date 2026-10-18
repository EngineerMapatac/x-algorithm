namespace rs xai_home_mixer_thrift.scored_candidate

struct ScoredCandidate {
  1: required i64 tweetId
  2: optional i64 viewerId
  3: optional i64 authorId
  6: optional double score
  7: optional string suggestType
  17: optional i64 requestTimeMs
  22: optional i64 sourceTweetId
  24: optional set<PredictionScore> predictionScores
}

struct PredictionScore {
  1: optional string name
  2: optional double score
}

struct ServedSearchCandidateList {
  1: required list<i64> tweetIds
  2: optional i64 viewerId
  3: optional i64 requestTimeMs
  4: optional i64 servedRequestId
  5: optional string rawQuery
}
