package com.twitter.botmaker.app.scarecrow.function

import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.types.{Symbol => BMSymbol}
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit1
import java.lang.{Long => JLong}
import java.util.{Set => JSet}
import scala.collection.JavaConverters._

object GetLongList extends FunctionUnit1[ScarecrowRuntime, BMSymbol, JSet[JLong]] {
  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description = "String ListName Return a set of long number based on the list name."
  override def arguments: Seq[String] = Seq[String]("1.Symbol ListName")
  override def examples: Seq[String] =
    Seq[String]("GetLongList(`LIST_NAME`)")

  override def evaluate(context: Context[ScarecrowRuntime], name: BMSymbol): JSet[JLong] = {
    context.getRuntime.fetcher20.getSimpleListLongValues(name.getValue)().asJava
  }
}
