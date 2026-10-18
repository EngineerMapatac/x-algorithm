package com.twitter.botmaker.app.scarecrow.legacy.function

import com.google.common.base.Optional
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.actioner.RateLimitedFunctionException
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerFunction
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.fetcher.Fetcher10
import com.twitter.botmaker.rule.RuleContext
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.Extractor
import com.twitter.botmaker.GeneralAction
import com.twitter.botmaker.MigrateContext
import com.twitter.botmaker.UserAction
import com.twitter.common.collections.Pair
import com.twitter.util.Function0
import com.twitter.util.Future

object RateLimited {
  val paramType: Type = Type.newGenericType
  val SIGNATURE: ASTNode.Signature =
    new ASTNode.Signature(
      ImmutableList.of(paramType, Type.LONG, Type.LONG),
      ImmutableList.of(Type.STRING, paramType),
      paramType
    )
}

@BotMakerFunction(
  argTypes = Array("T", "Long", "Long", "[String]", "[T]"),
  arguments = Array(
    "the function/action",
    "lookback period in seconds",
    "threshold",
    "Optional the real action name if it's diffrent than the funciton name",
    "Optional default value, otherwise throw exception"
  ),
  returnType = "T",
  description =
    "Ratelimit the realAction below the threshold, " +
      "call defaultAction when beyond threshold",
  actionLevel = ActionLevel.NO_ACTION,
  examples = Array("RateLimited(realAction, 60, 3000, \"\", defaultAction)")
)
class RateLimited(val exprText: String, val children: ImmutableList[ASTNode[_]])
    extends ASTNode[ScarecrowRuntime](exprText, children) {
  override def getCacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Never
  override def getSignature: ASTNode.Signature = RateLimited.SIGNATURE

  override def toExtractor: Extractor[ScarecrowRuntime] = {
    val childExtractors: ImmutableList[Extractor[_]] = buildExtractorsOfChildren
    val functionExtractor: Extractor[ScarecrowRuntime] =
      childExtractors.get(0).asInstanceOf[Extractor[ScarecrowRuntime]]
    val lookbackInSecondsExtractor: Extractor[ScarecrowRuntime] =
      childExtractors.get(1).asInstanceOf[Extractor[ScarecrowRuntime]]
    val thresholdExtractor: Extractor[ScarecrowRuntime] =
      childExtractors.get(2).asInstanceOf[Extractor[ScarecrowRuntime]]
    val aliasExtractorOption: Option[Extractor[ScarecrowRuntime]] =
      if (childExtractors.size > 3) {
        Some(childExtractors.get(3).asInstanceOf[Extractor[ScarecrowRuntime]])
      } else {
        None
      }
    val defaultValueExtractorOption: Option[Extractor[ScarecrowRuntime]] =
      if (childExtractors.size > 4) {
        Some(childExtractors.get(4).asInstanceOf[Extractor[ScarecrowRuntime]])
      } else {
        None
      }

    new Extractor[ScarecrowRuntime](this, childExtractors) {
      override def run(context: Context[ScarecrowRuntime]): Future[AnyRef] = {
        context.getRootContext match {
          case _: RuleContext[_] =>
            val runtime: ScarecrowRuntime = context.getRuntime
            val fetcher10: Fetcher10 = runtime.fetcher10
            val migrateContext: MigrateContext = runtime.getMigrateContext(context)
            val lookbackInSecondsFuture =
              context.run(lookbackInSecondsExtractor).asInstanceOf[Future[Long]]
            val thresholdFuture: Future[Long] =
              context.run(thresholdExtractor).asInstanceOf[Future[Long]]
            val actionNameFuture: Future[String] =
              aliasExtractorOption
                .map(context.run(_).asInstanceOf[Future[String]])
                .getOrElse(Future.value(functionExtractor.getExprText))
            Future
              .join(lookbackInSecondsFuture, thresholdFuture, actionNameFuture)
              .flatMap {
                case (lookbackInSeconds, threshold, actionName) =>
                  val actFunc = new Function0[Future[AnyRef]]() {
                    override def apply(): Future[AnyRef] = {
                      context.run(functionExtractor)
                    }
                  }
                  val defaultValueFunc =
                    new Function0[Pair[Optional[UserAction], Future[AnyRef]]]() {
                      override def apply(): Pair[Optional[UserAction], Future[AnyRef]] = {
                        defaultValueExtractorOption
                          .map { defaultValueExtractor =>
                            Pair.of(
                              Optional.of(UserAction.generalAction(new GeneralAction(actionName))),
                              context.run(defaultValueExtractor)
                            )
                          }.getOrElse {
                            throw new RateLimitedFunctionException(
                              s"ActionName: $actionName Threshold: $threshold"
                            )
                          }
                      }
                    }
                  fetcher10.getActioner.rateLimitedExtractor(
                    fetcher10,
                    migrateContext,
                    lookbackInSeconds,
                    threshold,
                    getClassName,
                    actionName,
                    actFunc,
                    defaultValueFunc
                  )
              }
          case _ => context.run(functionExtractor)
        }
      }
    }
  }
}
