package com.twitter.botmaker.app.scarecrow.legacy

import java.lang.{Boolean => JBoolean}
import java.lang.{Long => JLong}
import com.google.common.base.Optional
import com.twitter.botmaker._
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.util.Future
import com.twitter.botmaker.ASTNode.CacheLevel

object SetUserLabel
    extends FunctionUnit3O5[
      ScarecrowRuntime,
      Long,
      String,
      String,
      JLong,
      JBoolean,
      JBoolean,
      String,
      JLong,
      Future[Boolean]
    ] {
  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.PROD_TANGIBLE_IMPACT_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.Gizmoduck)
  override def description = "Set label on a given user"
  override def arguments: Seq[String] =
    Seq[String](
      "userId",
      "label type",
      "reason",
      "expiration AT time in ms; NOTE that this is NOT TTL (default = now + 7 days)",
      "whether force set label (default = FALSE)",
      "enableHoldbackExperiment (NOTE this is now unused and not respected)",
      "actor",
      "actorCreationMs"
    )

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    userId: Long,
    labelType: String,
    reason: String,
    expirationInMs: Option[JLong],
    forceSet: Option[JBoolean],
    enableHoldbackExperiment: Option[JBoolean],
    actor: Option[String],
    actorCreationMs: Option[JLong]
  ): Future[Boolean] = {
    if (context.getRuntime.systemConfigs.darkMode.get) {
      Future.value(true)
    } else {
      com.twitter.botmaker.compiler.v2.astnode.SetUserLabel
        .migrateEvaluate(
          context.getRuntime.fetcher10,
          context.getRuntime.getMigrateContext(context),
          userId,
          labelType,
          reason,
          Optional.fromNullable(expirationInMs.orNull),
          Optional.fromNullable(forceSet.orNull),
          Optional.fromNullable(actor.orNull),
          Optional.fromNullable(actorCreationMs.orNull)
        ).map(Boolean2boolean)
    }
  }
}
