package com.twitter.botmaker.rule.function

import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.Context
import com.twitter.botmaker.Extractor
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerFunction
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.rule.RuleContext
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.util.Future

object CacheSet {
  val retType: Type = Type.newGenericType
  val SIGNATURE: ASTNode.Signature =
    new ASTNode.Signature(ImmutableList.of(Type.STRING, retType), retType)
}

@BotMakerFunction(
  argTypes = Array("String", "T"),
  arguments = Array("key", "expensive-expression"),
  returnType = "T",
  description =
    """Update key and value in the in-memory loading cache of corresponding event context. The first argument is the cache key while the second expression is the expensive expression.""",
  actionLevel = ActionLevel.NO_ACTION
)
class CacheSet(val exprText: String, val children: ImmutableList[ASTNode[_]])
    extends ASTNode[TwitterRuntime](exprText, children) {
  override protected def getCacheLevel = CacheLevel.Never
  override def getSignature: ASTNode.Signature = CacheSet.SIGNATURE
  override def toExtractor: Extractor[TwitterRuntime] = {
    val keyNode: ASTNode[_] = getChildren.get(0)
    val keyExtractor: Extractor[_] = keyNode.toExtractor
    val exprNode: ASTNode[_] = getChildren.get(1)
    val exprExtractor: Extractor[_] = exprNode.toExtractor
    val children: ImmutableList[Extractor[_]] = ImmutableList.of(keyExtractor, exprExtractor)
    new Extractor[TwitterRuntime](this, children) {

      override def run(context: Context[TwitterRuntime]): Future[AnyRef] = {
        context.getRootContext match {
          case ruleContext: RuleContext[_] =>
            val valueFuture = context.run(exprExtractor.asInstanceOf[Extractor[TwitterRuntime]])
            context.run(keyExtractor.asInstanceOf[Extractor[TwitterRuntime]]).flatMap {
              case key: String =>
                ruleContext.eventContext.cachePut(CacheGet -> key, valueFuture)
                valueFuture
            }
          case _ =>
            context.run(exprExtractor.asInstanceOf[Extractor[TwitterRuntime]])
        }
      }
    }
  }
}
