package com.twitter.botmaker.app.publisher.function

import scala.collection.JavaConverters._
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.app.publisher.BotMakerPublisherRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerFunction
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.FunctionNode
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context

import java.util

@BotMakerFunction(
  argTypes = Array("String..."),
  arguments = Array("path components"),
  returnType = "botmaker arguments",
  description = "gets botmaker arguments",
  actionLevel = ActionLevel.NO_ACTION
)
class Arguments(exprText: String, children: ImmutableList[ASTNode[_]])
    extends FunctionNode[BotMakerPublisherRuntime](
      exprText,
      children
    ) {

  override protected def getCacheLevel = CacheLevel.Never

  override def getSignature: Signature = new Signature(
    Type.STRING,
    Type.mapOf(Type.STRING, Type.STRING)
  )

  override protected def apply(
    context: Context[BotMakerPublisherRuntime],
    args: util.List[AnyRef]
  ): AnyRef = {
    val runtime = context.getRuntime
    RuntimeDebugger
      .infos(
        runtime,
        args.asScala.toList collect { case x: String => x }
      )
      .asJava
  }
}
