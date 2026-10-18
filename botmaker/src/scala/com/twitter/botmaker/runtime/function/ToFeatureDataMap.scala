package com.twitter.botmaker.runtime.function

import com.google.common.collect.ImmutableList
import com.google.common.collect.ImmutableMap
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.common.FeatureDataConverter
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerFunction
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.FunctionNode1
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit1
import com.twitter.relevance.feature_store.thrift.FeatureData
import com.twitter.relevance.feature_store.thrift.FeatureValue

import java.util
import scala.collection.JavaConverters._

object ToFeatureDataMap {
  lazy val signature: Signature = new Signature(
    ImmutableList.of(Type.mapOf(Type.newGenericType, Type.newGenericType)),
    Type.mapOf(Type.newGenericType, Type.thriftOf(classOf[FeatureData]))
  )
}

@BotMakerFunction(
  argTypes = Array("Map[String, AnyRef]"),
  arguments = Array("A Map generated from inputfeatures, outputfeatures and derivedfeatures"),
  returnType = "",
  description = """Convert the Map[String, AnyRef] to Map[String, FeatureData]""",
  actionLevel = ActionLevel.NO_ACTION
)
class ToFeatureDataMap(exprText: String, children: ImmutableList[ASTNode[_]])
    extends FunctionNode1[TwitterRuntime, util.Map[AnyRef, AnyRef]](exprText, children) {
  override protected def getCacheLevel = CacheLevel.Event
  override def getSignature: Signature = ToFeatureDataMap.signature
  override protected def apply(
    context: Context[TwitterRuntime],
    target: util.Map[AnyRef, AnyRef]
  ): AnyRef = {

    val builder = ImmutableMap.builder.asInstanceOf[ImmutableMap.Builder[AnyRef, FeatureData]]

    for (entry <- target.entrySet.asScala) {
      builder.put(
        entry.getKey,
        FeatureDataConverter.typedObjectToFeatureData(entry.getValue)
      )
    }

    builder.build
  }
}

object GetFeatureValueTypeName extends FunctionUnit1[TwitterRuntime, FeatureValue, String] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description = "Get the BotMaker type name for the value stored a FeatureValue"
  override def arguments: Seq[String] = Nil

  override def evaluate(context: Context[TwitterRuntime], featureValue: FeatureValue): String = {
    val fieldType = FeatureDataConverter.javaFeatureValueFieldIdToTypeMapping(
      featureValue.getSetField().getThriftFieldId)
    fieldType.toString
  }
}
