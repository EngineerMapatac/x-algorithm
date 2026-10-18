package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit1
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.util.Future
import com.twitter.botmaker.compiler.v2.{astnode => bm1}
import java.util.{Map => JMap}
import java.util.{Set => JSet}
import com.twitter.botmaker.ASTNode.CacheLevel

object GetUrlResolutions
    extends FunctionUnit1[ScarecrowRuntime, JSet[String], Future[JMap[String, String]]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION

  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.Pinkstore)

  override def description = "The redirect chains for a set of urls."
  override def arguments: Seq[String] = Seq[String]("set of strings")
  override def examples: Seq[String] =
    Seq[String](
      "GetUrlResolutions(GetUrls(\"Prefix http://url1.com more text http://url2.com/somepath suffix\"))"
    )

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    urls: JSet[String]
  ): Future[JMap[String, String]] = bm1.GetUrlResolutions.migrateEvaluate(
    context.getRuntime.fetcher10,
    context.getRuntime.getMigrateContext(context),
    Future.value(urls)
  )
}
