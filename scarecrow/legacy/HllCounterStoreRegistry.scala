package com.twitter.botmaker.app.scarecrow.legacy

import java.util.concurrent.TimeUnit
import com.google.common.base.Optional
import com.twitter.botmaker._
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.spam.manhattan.v3.HLLCounterStore
import com.twitter.util.Duration
import com.twitter.util.Future
import com.twitter.botmaker.ASTNode.CacheLevel

object AddToHLL
    extends FunctionUnit4O4[
      ScarecrowRuntime,
      String,
      String,
      String,
      String,
      Long,
      Long,
      Long,
      Long,
      Future[java.util.Set[com.twitter.botmaker.ResponseCode]]
    ] {
  private val DefaultPrecision: Long = 60
  private val DefaultTtl: Long = 25

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.PROD_OTHER_ACTION
  override def downstreams: Set[DownstreamService] =
    Set(DownstreamServices.ManhattanBotmakerHllCounterStore)
  override def description =
    "Add an item to HLL store. The max allowed ttl for this data is 76 days. If the provided ttl argument has a higher value, it will be capped at 76 days."
  override def arguments: Seq[String] =
    Seq[String](
      "namespace",
      "topic",
      "id",
      "value",
      "timestampMS",
      "precision in seconds",
      "manhattan ttl in hours (max of 76 days)",
      "slice number'"
    )

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    namespace: String,
    topic: String,
    id: String,
    value: String,
    timestampMs: Option[Long],
    precision: Option[Long],
    ttl: Option[Long],
    sliceNumber: Option[Long]
  ): Future[java.util.Set[com.twitter.botmaker.ResponseCode]] = {

    com.twitter.botmaker.compiler.v2.astnode.AddToHLL.migrateEvaluate(
      context.getRuntime.fetcher10,
      context.getRuntime.getMigrateContext(context),
      Future.value(namespace),
      Future.value(topic),
      Future.value(id),
      Future.value(value),
      Future.value(Long.box(timestampMs.getOrElse(0L))),
      Future.value(Duration.fromSeconds(precision.getOrElse(DefaultPrecision).toInt)),
      Future.value(Duration.fromTimeUnit(ttl.getOrElse(DefaultTtl), TimeUnit.HOURS)),
      Future.value(Long.box(sliceNumber.getOrElse(HLLCounterStore.DEFAULT_SLICE_NUMBER)))
    )
  }
}

object GetHLLCount
    extends FunctionUnit4O3[
      ScarecrowRuntime,
      AnyRef,
      String,
      String,
      Long,
      Long,
      Long,
      Long,
      Future[Long]
    ] {
  private val DefaultPrecision: Long = 60

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] =
    Set(DownstreamServices.ManhattanBotmakerHllCounterStore)
  override def description =
    "Gets HLL count. Note that no item in the HHL store can live longer than 76 days."
  override def arguments: Seq[String] =
    Seq[String](
      "namespaces",
      "topic",
      "name",
      "lookback in seconds",
      "precision in seconds",
      "cache ttl in seconds",
      "slice number"
    )

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    namespacesObj: AnyRef,
    topic: String,
    name: String,
    lookback: Long,
    precision: Option[Long],
    cachingTtlSecs: Option[Long],
    sliceNumber: Option[Long]
  ): Future[Long] = {
    com.twitter.botmaker.compiler.v2.astnode.GetHLLCount
      .migrateEvaluate(
        context.getRuntime.fetcher10,
        context.getRuntime.getMigrateContext(context),
        Future.value(namespacesObj),
        Future.value(topic),
        Future.value(name),
        Future.value(Duration.fromSeconds(lookback.toInt)),
        Future.value(Duration.fromSeconds(precision.getOrElse(DefaultPrecision).toInt)),
        Future.value(Optional.fromNullable(cachingTtlSecs.map(Long.box).orNull)),
        Future.value(Long.box(sliceNumber.getOrElse(HLLCounterStore.DEFAULT_SLICE_NUMBER)))
      ).map(Long.unbox)
  }
}
