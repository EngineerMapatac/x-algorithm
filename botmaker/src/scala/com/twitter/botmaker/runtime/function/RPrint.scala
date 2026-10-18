package com.twitter.botmaker.runtime.function

import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerFunction
import com.twitter.botmaker.compiler.tuples.Pair
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.FunctionNode1O1
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context

@BotMakerFunction(
  argTypes = Array("Object", "[Long]"),
  arguments = Array("message", "limit per minute"),
  returnType = "Unit",
  description = "Ratelimited Log a message",
  actionLevel = ActionLevel.NO_ACTION
)
class RPrint(exprText: String, children: ImmutableList[ASTNode[_]])
    extends FunctionNode1O1[TwitterRuntime, AnyRef, Long](
      exprText,
      children
    ) {

  override def getSignature: Signature = new Signature(
    ImmutableList.of(Type.OBJECT),
    ImmutableList.of(Type.LONG),
    Type.UNIT
  )

  override protected def getCacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event

  override protected def apply(context: Context[TwitterRuntime], msg: AnyRef): AnyRef = {
    apply(context, msg, 1L)
  }

  override protected def apply(
    context: Context[TwitterRuntime],
    msg: AnyRef,
    limit: Long
  ): AnyRef = {
    context.getRuntime.logger.info(
      Pair.of(this, context.getRuntime.clock.nowMillis() % limit),
      s"go/bot/${context.getRuntime.name}/${context.getTrackingId} log: ${String.valueOf(msg)}"
    )
    unit()
  }
}
