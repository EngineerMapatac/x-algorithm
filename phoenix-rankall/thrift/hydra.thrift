namespace java com.twitter.strato.columns.hydra.thriftjava
namespace py gen.twitter.strato.columns.hydra

struct PhoenixRankAllObject {
  1: i64 postId (personalDataType = 'TweetId')
  2: i64 authorId
  3: string indexName
  100: optional list<i64> topicEntityIds
}

struct EngagementCount {
  1: optional i64 retweetCount
  2: optional i64 replyCount
  3: optional i64 favoriteCount
  4: optional i64 quoteCount
  5: optional i64 bookmarkCount
  6: optional i64 viewCount
  7: optional i64 notInterestedInCount
  8: optional i64 reportCount
  9: optional i64 blockCount
}

struct MultiModalEmbeddings {
  1: optional list<double> mmEmbV1
  2: optional list<double> mmEmbV3
}

struct PhoenixRankAllCandidateFlat {
  1: i64 postId (personalDataType = 'TweetId')
  2: i64 authorId
  3: bool hasVideo
  4: bool hasImage
  5: i64 authorFollowersCount
  6: optional EngagementCount engagementCount
  8: optional i64 videoDurationMs
  9: optional MultiModalEmbeddings mmEmbs
  100: optional string indexName
}