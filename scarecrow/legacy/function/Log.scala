package com.twitter.botmaker.app.scarecrow.legacy.function

import scala.collection.JavaConverters._
import com.google.common.collect.ImmutableList
import com.twitter.botmaker._
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerFunction
import com.twitter.botmaker.compiler.types.ScalaType.ofType
import com.twitter.botmaker.compiler.exceptions.FunctionFailure
import com.twitter.botmaker.compiler.tuples.Pair
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.DerivedFeatureCall
import com.twitter.botmaker.function.conversion.ToPair
import com.twitter.botmaker.rule.function.ResponseCodes
import com.twitter.util.Future

object Log {
  val signature: ASTNode.Signature =
    new ASTNode.Signature(
      ImmutableList.of[Type],
      Type.pairOf(Type.STRING, Type.OBJECT),
      Type.setOf(ofType[ResponseCode])
    )
}

@BotMakerFunction(
  argTypes = Array("[Pair<String, Object>...]"),
  arguments = Array(
    "feature name",
    "feature value",
    "remaining feature name/value pairs"
  ),
  returnType = "Set<ResponseCode>",
  description =
    """Adds the given feature name / expression pairs as derived features to the BotMaker log.""",
  actionLevel = ActionLevel.NO_ACTION,
  examples = Array(
    """Log("hi", 998, "yoo", 42)""",
    """Log("FeatureA", 15, "FeatureB", "1.2.3.4", "FeatureC", Sum(2.0, + 2.0))"""
  )
)
class Log(val exprText: String, val children: ImmutableList[ASTNode[_]])
    extends ASTNode[ScarecrowRuntime](exprText, ToPair.convertNodesToPairs(children, 0)) {

  override def getSignature: ASTNode.Signature = Log.signature

  override protected def getCacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event

  override def toExtractor: Extractor[ScarecrowRuntime] = {

    val childASTNodes = getChildren
    val childExtractors = buildExtractorsOfChildren

    new Extractor[ScarecrowRuntime](this, childExtractors) {
      override def run(context: Context[ScarecrowRuntime]): Future[AnyRef] = {

        Future collect {
          childExtractors.asScala.map(e => context.run(e.asInstanceOf[Extractor[ScarecrowRuntime]]))
        } map { seq =>
          val list = seq.asJava
          for (i <- (0 until childExtractors.size)) {
            val pair = list.get(i).asInstanceOf[Pair[_, _]]
            val fn = pair.getFirst.asInstanceOf[String]
            val fv = pair.getSecond
            val ft = childASTNodes.get(i).getChildren.get(1).getReturnType
            try {
              context.addOutputFeature(
                DerivedFeatureCall.NAMESPACE,
                fn,
                ft,
                fv
              )
            } catch {
              case t: Throwable => new FunctionFailure(Log.this, context.getStackFrames, t)
            }
          }
          ResponseCodes.None
        }
      }
    }
  }
}
