package com.twitter.tweet_entity_service.columns

import com.google.inject.Inject
import com.google.inject.Singleton
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.stitch.Arrow
import com.twitter.tseng.withholding.thriftscala.TakedownReason
import com.twitter.tweet_entity_service.columns.internal.BaseTweetFieldColumn
import com.twitter.tweet_entity_service.model.TweetFieldKeyType
import com.twitter.tweet_entity_service.transforms.TakedownReasonsTransform
import com.twitter.tweet_entity_service.transforms.TweetReposTransform

@Singleton
class TakedownReasonsColumn @Inject() (
  statsReceiver: StatsReceiver,
  tweetReposTransform: TweetReposTransform,
  takedownReasonsTransform: TakedownReasonsTransform)
    extends BaseTweetFieldColumn[Seq[TakedownReason]](
      name = "takedownReasons",
      tweetFieldKeyType = TweetFieldKeyType.TakedownReasons,
      statsReceiver = statsReceiver,
      extractField = Arrow.map(_.tweetFields.map(_.takedownReasons)),
      repositoryArrow = tweetReposTransform.get,
      isoMiddleWares = Seq(takedownReasonsTransform.get)
    )
