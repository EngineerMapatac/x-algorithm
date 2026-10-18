package com.twitter.botmaker.rule.function

import com.google.common.collect.ImmutableList
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.Extractor
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerFunction
import com.twitter.botmaker.compiler.exceptions.RuntimeFailure
import com.twitter.botmaker.rule.RuleContext
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.util.Future

object CacheGet {
  val retType: Type = Type.newGenericType
  val SIGNATURE: ASTNode.Signature =
    new ASTNode.Signature(ImmutableList.of(Type.STRING, retType), retType)
}

@BotMakerFunction(
  name = Array("CacheGet"),
  argTypes = Array("String", "T"),
  arguments = Array("key", "expensive-expression"),
  returnType = "T",
  description =
    """Cache expensive expressions in the in-memory loading cache of corresponding event context. The first argument is the cache key while the second expression is the expensive expression.""",
  actionLevel = ActionLevel.NO_ACTION
)
class CacheGet(exprText: String, children: ImmutableList[ASTNode[_]])
    extends ASTNode[TwitterRuntime](exprText, children) {

  override def getSignature: ASTNode.Signature = CacheGet.SIGNATURE
  override protected def getCacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def computeFingerprintScope(currentScope: Long): Long = ASTNode.CacheLevel.Event.scope

  override def toExtractor: Extractor[TwitterRuntime] = {
    val keyNode = getChildren.get(0).asInstanceOf[ASTNode[TwitterRuntime]]
    val keyExtractor = keyNode.toExtractor

    val exprNode = getChildren.get(1).asInstanceOf[ASTNode[TwitterRuntime]]
    val exprExtractor = exprNode.toExtractor

    val children = ImmutableList
      .of(
        keyExtractor,
        exprExtractor
      )
      .asInstanceOf[ImmutableList[Extractor[_]]]

    new Extractor[TwitterRuntime](this, children) {

      override def run(context: Context[TwitterRuntime]): Future[AnyRef] = {
        context.getRootContext match {
          case ruleContext: RuleContext[TwitterRuntime] =>
            context.run(keyExtractor) flatMap {
              case k: String =>
                ruleContext.eventContext.cacheGet(CacheGet -> k) {
                  context.run(exprExtractor)
                } match {
                  case f: Future[AnyRef] =>
                    f
                  case _ =>
                    Future.exception(new RuntimeFailure("corrupted data in event cache"))
                }
            }
          case _ =>
            context.run(exprExtractor)
        }
      }
    }
  }
}

@BotMakerFunction(
  name = Array("CacheGetUpdate"),
  argTypes = Array("String", "T"),
  arguments = Array("key", "expensive-expression"),
  returnType = "T",
  description =
    """CacheGetUpdate provides the same caching functions as CacheGet with one crucial difference: CacheGetUpdate is not cacheable so it is always executed. CacheGetUpdate can read the cached value updated by CacheSet function.""",
  actionLevel = ActionLevel.NO_ACTION
)
class CacheGetUpdate(exprText: String, children: ImmutableList[ASTNode[_]])
    extends CacheGet(exprText, children) {

  override def getSignature: ASTNode.Signature = CacheGet.SIGNATURE
  override protected def getCacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Never
  override def computeFingerprintScope(currentScope: Long): Long = ASTNode.CacheLevel.Never.scope
}
