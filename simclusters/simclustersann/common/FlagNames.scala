package com.twitter.simclustersann.common

object FlagNames {

  final val ServiceTimeout = "service.timeout"
  final val DarkTrafficFilterDeciderKey = "thrift.dark.traffic.filter.decider_key"

  final val CacheDest = "cache_module.dest"
  final val CacheTimeout = "cache_module.timeout"
  final val CacheAsyncUpdate = "cache_module.async_update"

  final val DisableWarmup = "warmup.disable"
  final val NumberOfThreads = "warmup.thread_number"
  final val RateLimiterQPS = "warmup.rate_limiter_qps"

  final val MaxTopTweetPerCluster = "sim_clusters.ann.max_top_tweets_per_cluster"

}
