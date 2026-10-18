package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit1
import com.twitter.util.Future
import java.lang.{Long => JLong}
import java.util.{Set => JSet}
import java.util.{List => JList}

object GetTweetIdsForSlugs
    extends FunctionUnit1[ScarecrowRuntime, JList[String], Future[JSet[JLong]]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.SlugToTweetIdStore)
  override def description =
    "Given a list of t.co slugs, provide ids of tweets that contain the given slugs."
  override def arguments: Seq[String] = Seq[String]("list of t.co slugs")
  override def examples: Seq[String] =
    Seq[String]("GetTweetIdsForSlugs(List(\"DboS57QbBv\",\"MTwP58LwvM\"))")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    slugs: JList[String]
  ): Future[JSet[JLong]] = context.getRuntime.fetcher10.getTweetIdsForSlugs(slugs)

}
