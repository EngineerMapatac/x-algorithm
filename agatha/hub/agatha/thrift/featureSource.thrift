namespace java com.twitter.hub.agatha.thriftjava

enum FeatureSource {
  FollowGraph = 1,
  FavGraph = 2,
  RetweetGraph = 3,
  ReplyGraph = 4,
  MentionGraph = 18,
  QuoteGraph = 19,
  PhonebookGraph = 5,
  EmailGraph = 6,
  BlockGraph = 7,
  MuteGraph = 8,
  AbuseReportGraph = 9,
  ContributorGraph = 10,
  LegitTattleGraph = 22,
  NonLegitTattleNoBlocksGraph = 23,
  DirectMessageGraph = 24,

  HashtagOriginal = 11,
  HashtagRetweet = 12,
  ListFollowers = 14,
  ListMembership = 15,
  StringTokensInBio = 16,
  RetweetedTweet = 20,
  FavedTweet = 21,

  CombinedUser = 17
}
