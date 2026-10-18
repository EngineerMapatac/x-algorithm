package com.twitter.botmaker.app.scarecrow.legacy

import java.lang.{Long => JLong}
import java.util.{Set => JSet}
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit3O1
import com.twitter.botmaker.FunctionUnit3O2
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.rule.function.ResponseCodes
import com.twitter.util.Future
import com.twitter.botmaker.ASTNode.CacheLevel

object GetFeatureCountLite
    extends FunctionUnit3O2[
      ScarecrowRuntime,
      String,
      String,
      Long,
      Boolean,
      Boolean,
      Future[Long]
    ] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.LimiterService)
  override def description: String =
    """Feature count over the given lookback period for Feature created using 
      | IncrementFeatureCountLite, performs best with short lookback intervals. WARNING: This
      | function by default only reads from the local DC, which will often return incorrect 
      | counts as many things happen across DCs. It is recommended to use other count functions 
      | instead, unless there is a strong need for an accurate very-short duration
      | counter""".stripMargin
  override def arguments: Seq[String] =
    Seq[String](
      "the feature",
      "ID for which to retrieve value",
      "lookback period in seconds",
      "skip the 1-second cache protecting the downstream service. Please notify Health Core" +
        " Infrastructure team before setting this argument to TRUE. Default FALSE",
      "whether to read from all DCs. Default FALSE"
    )
  override def examples: Seq[String] =
    Seq[String]("GetFeatureCountLite(\"testFeature\", \"testId\", 5)")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    featureName: String,
    id: String,
    lookback: Long,
    skipCache: Option[Boolean],
    readFromAllDCs: Option[Boolean]
  ): Future[Long] = {
    context.getRuntime.fetcher10
      .getFeatureCountLite(
        context.getRuntime.getMigrateContext(context),
        Future.value(featureName),
        Future.value(id),
        Future.value(lookback),
        Future.value(boolean2Boolean(skipCache.getOrElse(false))),
        Future.value(boolean2Boolean(readFromAllDCs.getOrElse(false)))
      ).map(Long2long)
  }
}

object IncrementFeatureCountLite
    extends FunctionUnit3O1[ScarecrowRuntime, String, String, JLong, Boolean, Future[
      JSet[com.twitter.botmaker.ResponseCode]
    ]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Never
  override def actionLevel: ActionLevel = ActionLevel.PROD_OTHER_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.LimiterService)
  override def description: String =
    """Way to increment a value for a feature by the given amount that offers more accuracy
      | but shorter retention time (1 day). WARNING: This function is likely paired with
      | GetFeatureCountLite which by default only reads from the local DC, which will often
      | return incorrect counts as many things happen across DCs. It is recommended to use other
      | count functions instead, unless there is a strong need for an accurate very-short
      | duration counter""".stripMargin
  override def arguments: Seq[String] =
    Seq[String](
      "the feature",
      "ID for which the feature has to be incremented",
      "value to increment by",
      "whether to write to all DCs. Default FALSE"
    )
  override def examples: Seq[String] =
    Seq[String]("IncrementFeatureCountLite(\"testFeature\", \"testId\", 5)")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    featureName: String,
    id: String,
    incrementBy: JLong,
    writeToAllDCs: Option[Boolean]
  ): Future[JSet[com.twitter.botmaker.ResponseCode]] = {
    context.getRuntime.fetcher10
      .incrementFeatureCountLite(
        Future.value(featureName),
        Future.value(id),
        Future.value(incrementBy),
        Future.value(boolean2Boolean(writeToAllDCs.getOrElse(false)))
      ).map(_ => ResponseCodes.None)
  }
}
