package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker._
import com.twitter.util.Future
import java.util.{List => JList}
import java.util.{Map => JMap}
import java.util.{Optional => JOptional}
import java.util.{Set => JSet}

object GetReputationV2
    extends FunctionUnit1[ScarecrowRuntime, JMap[String, String], Future[
      JMap[String, JMap[String, String]]
    ]] {
  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.ManhattanUrlReputation)
  override def description =
    "The function returns a reputation map (Keys: verdict, source, creator) for url redirect chains."
  override def arguments: Seq[String] =
    Seq[String]("a map of url string to its redirect chain. Hops are separated by space")
  override def examples: Seq[String] =
    Seq[String](
      "GetReputationV2(GetUrlResolutions(Set(\"http://url1.com, http://url2.com/somepath\")))"
    )

  override def evaluate(
    context: Context[ScarecrowRuntime],
    resolutions: JMap[String, String]
  ): Future[JMap[String, JMap[String, String]]] = {
    com.twitter.botmaker.compiler.v2.astnode.GetReputationV2.migrateEvaluate(
      context.getRuntime.fetcher10,
      Future.value(resolutions)
    )
  }
}

object GetUrlVerdict extends FunctionUnit1[ScarecrowRuntime, String, Future[String]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.ManhattanUrlReputation)
  override def description = "Gets the verdict for a given url"
  override def arguments: Seq[String] = Seq[String]("url url", "verdict verdict")
  override def examples: Seq[String] = Seq[String]("GetUrlVerdict(\"http://www.url1.com/\")")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    url: String
  ): Future[String] = {
    com.twitter.botmaker.compiler.v2.astnode.GetUrlVerdict
      .migrateEvaluate(
        context.getRuntime.fetcher10,
        Future.value(url)
      )
  }
}

object GetUrlVerdictForSingleChain
    extends FunctionUnit1[ScarecrowRuntime, JList[String], Future[String]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.ManhattanUrlReputation)
  override def description = "The verdict (GOOD/BAD/UNKNOWN) for a single url redirect chain."
  override def arguments: Seq[String] =
    Seq[String]("a list of strings representing the redirection chain of a single url")
  override def examples: Seq[String] = Seq[String]("GetUrlVerdictForSingleChain(resolutionChain)")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    redirectChain: JList[String]
  ): Future[String] = {
    com.twitter.botmaker.extractor.function.GetUrlVerdictForSingleChain.migrateEvaluate(
      context.getRuntime.fetcher10,
      Future.value(redirectChain)
    )
  }

}

object GetUrlVerdictForSingleChainV2
    extends FunctionUnit1[ScarecrowRuntime, JList[String], Future[JMap[String, String]]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.ManhattanUrlReputation)
  override def description =
    "The verdict map for a single url redirect chain. The verdict map contains the keys - verdict, source and creator"
  override def arguments: Seq[String] =
    Seq[String]("a list of strings representing the redirection chain of a single url")
  override def examples: Seq[String] =
    Seq[String]("GetUrlVerdictForSingleChainV2(List(\"twitter.com\"))")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    chain: JList[String]
  ): Future[JMap[String, String]] = {
    com.twitter.botmaker.compiler.v2.astnode.GetUrlVerdictForSingleChainV2.migrateEvaluate(
      context.getRuntime.fetcher10,
      Future.value(chain)
    )
  }
}

object GetUrlVerdictV2
    extends FunctionUnit1[ScarecrowRuntime, String, Future[JMap[String, String]]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.ManhattanUrlReputation)
  override def description =
    "Get verdict map for a url. Map contains the keys - verdict, source and creator"
  override def arguments: Seq[String] = Seq[String]("String url")
  override def examples: Seq[String] = Seq[String]("GetUrlVerdictV2(\"twitter.com\")")

  override def evaluate(
    context: Context[ScarecrowRuntime],
    url: String
  ): Future[JMap[String, String]] = {
    com.twitter.botmaker.compiler.v2.astnode.GetUrlVerdictV2.migrateEvaluate(
      context.getRuntime.fetcher10,
      Future.value(url)
    )
  }
}

object DeleteUrlVerdictIf
    extends FunctionUnit3O2[ScarecrowRuntime, String, String, String, String, String, Future[
      JSet[com.twitter.botmaker.ResponseCode]
    ]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.PROD_TANGIBLE_IMPACT_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.ManhattanUrlReputation)
  override def description =
    "Delete a verdict from the reputation store, only if it matches existingVerdict"
  override def arguments: Seq[String] =
    Seq[String]("String entity", "String matchingType", "String existingVerdict")
  override def examples: Seq[String] =
    Seq[String](
      "DeleteUrlVerdictIf(\"a.b.c.com\", \"DOMAIN\", \"BAD\", \"BOTMAKER\", \"123\")",
      "DeleteUrlVerdictIf(\"a.b.c.com\", \"DOMAIN\", \"BAD\", \"BOTMAKER\")",
      "DeleteUrlVerdictIf(\"a.b.c.com\", \"DOMAIN\", \"BAD\")"
    )

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    entity: String,
    matchingType: String,
    verdict: String,
    source: Option[String],
    creator: Option[String]
  ): Future[JSet[com.twitter.botmaker.ResponseCode]] = {
    com.twitter.botmaker.extractor.action.DeleteUrlVerdictIf.migrateEvaluate(
      context.getRuntime.fetcher10,
      Future.value(entity),
      Future.value(matchingType),
      Future.value(verdict),
      JOptional.ofNullable(source.orNull),
      JOptional.ofNullable(creator.orNull)
    )
  }
}

object WriteUrlVerdictsV2
    extends FunctionUnit5O2[ScarecrowRuntime, JSet[
      String
    ], String, Long, String, String, String, String, Future[
      JSet[com.twitter.botmaker.ResponseCode]
    ]] {

  private val DEFAULT_SOURCE = "botmaker"

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.PROD_TANGIBLE_IMPACT_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.ManhattanUrlReputation)
  override def description =
    "Writes the specified verdict to the set of URLs in url reputation store."
  override def arguments: Seq[String] =
    Seq[String](
      "a set of urls extracted.",
      "The string verdict - must be either BAD/GOOD/WHITELIST/KNOWN_UNKNOWN",
      "The TTL in Seconds",
      "The string matching type must be either EXACT_URL_MATCH/ URL_WITHOUT_PARAMETER / DOMAIN",
      "note",
      "source",
      "creator"
    )
  override def examples: Seq[String] =
    Seq[String](
      "WriteUrlVerdictsV2(GetUrls(\"Prefix http://url1.com more text http://url2.com/somepath suffix\"), \"known_unknown\", 1000, \"EXACT_URL_MATCH\", \"note\")",
      "WriteUrlVerdictsV2(GetUrls(\"Prefix http://url1.com more text http://url2.com/somepath suffix\"), \"GOod\", 1000, \"DOMAIN\", \"a note \", \"a source \", \"a creator \")"
    )

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    urls: JSet[String],
    verdict: String,
    ttl: Long,
    matchingType: String,
    note: String,
    source: Option[String],
    creator: Option[String]
  ): Future[JSet[com.twitter.botmaker.ResponseCode]] = {
    val migrateContext = context.getRuntime.getMigrateContext(context)
    com.twitter.botmaker.compiler.v2.astnode.WriteUrlVerdictsV2
      .migrateEvaluate(
        context.getRuntime.fetcher10,
        migrateContext,
        Future.value(urls),
        Future.value(verdict),
        Future.value(ttl),
        Future.value(matchingType),
        Future.value(note),
        Future.value(source.getOrElse(DEFAULT_SOURCE)),
        Future.value(creator.getOrElse(String.valueOf(migrateContext.getRuleBeingEvaluated.getId)))
      )
  }
}
