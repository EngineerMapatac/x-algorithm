package com.twitter.botmaker.app.scarecrow.legacy

import java.util.{List => JList}
import com.google.common.base.Optional
import com.twitter.botmaker.FunctionUnit2O1
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.util.Future
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.ASTNode.CacheLevel

object GetSlugsForUrl
    extends FunctionUnit2O1[ScarecrowRuntime, String, Long, Long, Future[JList[String]]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.UrlToSlugStore)
  override def description =
    "Given a URL, retrieves t.co slugs that are the first hop of the the resolution chain that includes the given URL.  The MatchingType parameter specifies the type of url_reputation.MatchingType to be applied when looking up the slugs."
  override def arguments: Seq[String] =
    Seq[String](
      "URL",
      "url_reputation MatchingType thrift enum value",
      "look back days, default 14 if not set"
    )
  override def examples: Seq[String] =
    Seq[String]("GetSlugsForUrl(\"http://www.test.com/DboS57QbBv\", 3)")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    url: String,
    matchingType: Long,
    lookbackDays: Option[Long]
  ): Future[JList[String]] = {
    val lookbackDaysOptional: Optional[Integer] = lookbackDays match {
      case None => Optional.absent[Integer]
      case Some(days) =>
        try {
          Optional.of(Math.toIntExact(days))
        } catch {
          case _: ArithmeticException => Optional.absent[Integer]
        }
    }

    com.twitter.botmaker.extractor.function.GetSlugsForUrl
      .migrateEvaluate(
        context.getRuntime.fetcher10,
        Future.value(url),
        Future.value(matchingType),
        Future.value(lookbackDaysOptional)
      )
  }

}
