package com.twitter.tweet_entity_service.transforms

import com.google.inject.Inject
import com.google.inject.Singleton
import com.google.inject.name.Named
import com.twitter.gizmoduck.thriftscala.LookupContext
import com.twitter.gizmoduck.thriftscala.QueryFields
import com.twitter.stitch.Arrow
import com.twitter.stitch.gizmoduck.Gizmoduck
import com.twitter.tseng.withholding.thriftscala.TakedownReason
import com.twitter.tweet_entity_service.model.TweetEnvelope
import com.twitter.takedown.util.TakedownReasons

@Singleton
class TakedownReasonsTransform @Inject() (
  @Named("UserVisibilityGizmoduck") gizmoduck: Gizmoduck,
  tbirdManhattanTransform: TbirdManhattanTransform) {

  private val userTakedownReasons: Arrow[TweetEnvelope, Seq[TakedownReason]] =
    Arrow
      .identity[TweetEnvelope]
      .map(_.tweetFields.flatMap(_.coreData).flatMap(_.userId))
      .andThen {
        Arrow.option {
          Arrow
            .flatMap { userId: Long =>
              gizmoduck
                .getById(userId, Set(QueryFields.Takedowns), LookupContext())
                .map(_.user.flatMap(_.takedowns.map(TakedownReasons.userTakedownsToReasons)))
                .map(_.toSeq.flatten)
            }
        }
      }
      .map(_.getOrElse(Seq.empty))

  private val tweetTakedownReasons: Arrow[TweetEnvelope, Seq[TakedownReason]] =
    Arrow
      .identity[TweetEnvelope]
      .andThen {
        Arrow
          .choose(
            Arrow.Choice.when(
              _.tweetFields.exists(_.takedownReasons.nonEmpty),
              Arrow.identity[TweetEnvelope]
            ),
            Arrow.Choice.otherwise(tbirdManhattanTransform.get)
          )
      }
      .map(_.tweetFields.map(_.takedownReasons))
      .map(_.toSeq.flatten)

  private val fetchIfHasTakedown: Arrow.Choice[TweetEnvelope, TweetEnvelope] =
    Arrow.Choice.when(
      _.tweetFields.flatMap(_.hasTakedown).getOrElse(false),
      Arrow
        .zipWithArg(Arrow.join(tweetTakedownReasons, userTakedownReasons))
        .map {
          case (tweetEnvelope, (tweetTakedownReasons, userTakedownReasons)) =>
            val combinedTakedownReasons = tweetTakedownReasons ++ userTakedownReasons
            tweetEnvelope.copy(
              tweetFields = tweetEnvelope.tweetFields.map(
                _.copy(
                  takedownReasons = combinedTakedownReasons,
                  tweetypieOnlyTakedownReasons = tweetTakedownReasons
                )
              )
            )
        }
    )

  val get: Arrow.Iso[TweetEnvelope] =
    Arrow.choose(fetchIfHasTakedown, Arrow.Choice.otherwise(Arrow.identity))
}
