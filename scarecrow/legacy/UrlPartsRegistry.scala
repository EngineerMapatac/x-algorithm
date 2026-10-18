package com.twitter.botmaker.app.scarecrow.legacy

import java.util.{Set => JSet}
import com.twitter.botmaker.FunctionUnit1
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.ASTNode.CacheLevel

object GetUrls extends FunctionUnit1[ScarecrowRuntime, String, JSet[String]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description = "The set of URLs extracted from the given text string."
  override def arguments: Seq[String] = Seq[String]("text string")
  override def examples: Seq[String] =
    Seq[String]("GetUrls(\"Hey, http://morefollowers.com.ua or followme.info or followme.info\")")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    text: String
  ): JSet[String] = {
    com.twitter.botmaker.extractor.function.GetUrls
      .migrateEvaluate(
        context.getRuntime.fetcher10,
        text
      )
  }
}
