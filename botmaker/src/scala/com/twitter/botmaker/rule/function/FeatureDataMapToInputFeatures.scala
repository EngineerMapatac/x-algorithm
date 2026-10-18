package com.twitter.botmaker.rule.function

import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit1
import com.twitter.botmaker.rule.BotMakerRuntime
import com.twitter.relevance.feature_store.thrift.FeatureData
import java.util.{Map => JMap}
import scala.collection.JavaConverters._

object FeatureDataMapToInputFeatures
    extends FunctionUnit1[BotMakerRuntime[_], JMap[String, FeatureData], JMap[String, AnyRef]] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description: String = """Convert FeatureData Map into a property bag"""
  override def arguments: Seq[String] = Seq("Feature data map")

  override def evaluate(
    context: Context[BotMakerRuntime[_]],
    featureDataMap: JMap[String, FeatureData]
  ): JMap[String, AnyRef] = {
    featureDataMap.asScala.filter {
      case (k, v) => v != null && v.featureValue != null
    } map {
      case (k, v) => k -> v.featureValue.getFieldValue
    } asJava
  }
}
