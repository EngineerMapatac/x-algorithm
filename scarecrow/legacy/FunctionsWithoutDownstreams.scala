package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker._
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.app.scarecrow.migrate.MigrateConfig20
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.types.{Symbol => BMSymbol}
import com.twitter.twittertext.Extractor
import java.util.{List => JList}

object SQRLVerdictTag extends FunctionUnit1[ScarecrowRuntime, BMSymbol, Unit] {
  override def cacheLevel: CacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description = "A function hint to group rules together using the same verdict." +
    "Note this function only works in rules. If used in a derived feature, " +
    "this function hint will be ignored. Use `SQRLVerdictMap` to retrieve the group."
  override def arguments: Seq[String] = Seq("verdict used to group rules together")
  override def examples: Seq[String] = Seq("SQRLVerdictTag(`BLACKLIST`)")
  override def evaluate(context: Context[ScarecrowRuntime], verdict: BMSymbol): Unit = ()
}

object GetMentionedScreenNames extends FunctionUnit1[ScarecrowRuntime, String, JList[String]] {
  override def cacheLevel: CacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description = "Returns the mentioned screen names in a given piece of text."
  override def arguments: Seq[String] = Seq("Text to extract screen names from")
  override def examples: Seq[String] =
    Seq("GetMentionedScreenNames(\"Hey, I am a spammer @dickc @ev\")")

  val Extractor: ThreadLocal[Extractor] = new ThreadLocal[Extractor]() {
    override protected def initialValue = new Extractor
  }

  override def evaluate(context: Context[ScarecrowRuntime], text: String): JList[String] = {
    Extractor.get.extractMentionedScreennames(text)
  }
}

object IsTestUser extends FunctionUnit1[ScarecrowRuntime, Long, Boolean] {

  override def cacheLevel: CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description =
    "Check if the user is an internal Twitter test user such as '@cas_2050004934'."
  override def arguments: Seq[String] = Seq[String]("the user ID")
  override def examples: Seq[String] = Seq[String]("IsTestUser(2050004934)")

  override def evaluate(context: Context[ScarecrowRuntime], userId: Long): Boolean = {

    ExemptionUtil.isTestUser(MigrateConfig20(context.getRuntime), userId)
  }
}
