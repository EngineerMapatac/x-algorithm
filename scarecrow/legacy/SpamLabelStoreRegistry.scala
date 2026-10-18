package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.rule.function.ResponseCodes
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit2
import com.twitter.botmaker.FunctionUnit3O2
import com.twitter.botmaker.ResponseCode
import com.twitter.util.Duration
import com.twitter.util.Future
import java.util.concurrent.TimeUnit
import java.util.{Set => JSet}

object GetLabel extends FunctionUnit2[ScarecrowRuntime, String, String, Future[String]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.ManhattanLabelstore)
  override def description: String =
    """Get label for a given key with the given namespace from the label store.
      |Note that the slash character (/) is not allowed in the namespace.
      |If a label is not found, an empty string is returned.
    """.stripMargin
  override def arguments: Seq[String] =
    Seq[String]("namespace for the key", "the key to get label for")
  override def examples: Seq[String] = Seq[String]("GetLabel(\"testLabelNamespace\", \"testKey\")")

  override def evaluate(
    context: Context[ScarecrowRuntime],
    namespace: String,
    key: String
  ): Future[String] = {
    if (namespace.contains('/')) {
      Future.exception(
        new Exception(
          "The slash character (/) is not allowed in the namespace argument of GetLabel"))
    } else {
      context.getRuntime.fetcher10.getLabel(
        context.getRuntime.getMigrateContext(context),
        Future.value(namespace),
        Future.value(key)
      )
    }
  }
}

class SetLabel
    extends FunctionUnit3O2[ScarecrowRuntime, String, String, String, Long, Boolean, Future[
      JSet[ResponseCode]
    ]] {

  private val DefaultTtlSeconds: Long = Duration.apply(26, TimeUnit.HOURS).inLongSeconds

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.PROD_OTHER_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.ManhattanLabelstore)
  override def description: String =
    """Set label for a given namespace and key in the label store.
      |Note that the slash character (/) is not allowed in the namespace.
    """.stripMargin

  override def arguments: Seq[String] =
    Seq[String](
      "namespace of the key",
      "the key",
      "label to set for the key",
      "ttl in secs = 26 * 3600",
      "whether update Twemcache = FALSE"
    )
  override def examples: Seq[String] =
    Seq[String](
      "SetLabel(\"testLabelNamespace\", \"testKey\", \"testLabel\", 233, TRUE)",
      "SetLabelWithTtl(\"testLabelNamespace\", \"testKey\", \"testLabel\", 60)",
      "SetLabel(\"testLabelNamespace\", \"testKey\", \"testLabel\")"
    )

  override def evaluate(
    context: Context[ScarecrowRuntime],
    namespace: String,
    key: String,
    label: String,
    ttlSecs: Option[Long],
    updateCache: Option[Boolean]
  ): Future[JSet[ResponseCode]] = {
    if (namespace.contains('/')) {
      Future.exception(
        new Exception(
          "The slash character (/) is not allowed in the namespace argument of SetLabel"))
    } else {
      context.getRuntime.fetcher10
        .setLabel(
          context.getRuntime.getMigrateContext(context),
          Future.value(namespace),
          Future.value(key),
          Future.value(label),
          Future.value(ttlSecs.getOrElse(DefaultTtlSeconds).toInt),
          Future.value(Boolean.box(updateCache.getOrElse(false)))
        ).map(_ => ResponseCodes.None)
    }
  }
}

object SetLabel extends SetLabel

object SetLabelWithTtl extends SetLabel
