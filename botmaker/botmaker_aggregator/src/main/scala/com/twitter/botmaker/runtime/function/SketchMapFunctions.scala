package com.twitter.botmaker.runtime.function

import com.twitter.algebird_internal.{thriftscala => t}
import com.twitter.botmaker._
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.runtime.config.SketchMapAggregator

object SketchMapFrequence extends FunctionUnit2[TwitterRuntime, t.SketchMap, String, Long] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def evaluate(
    context: Context[TwitterRuntime],
    sketchMap: t.SketchMap,
    item: String
  ): Long = {
    val sm = SketchMapAggregator.injection.invert(sketchMap).get
    SketchMapAggregator.monoid.frequency(sm, item)
  }

  override def description: String = "Gets frequence of an item in SketchMap"

  override def arguments: Seq[String] = Seq(
    "SketchMap thrift"
  )
}

object SketchMapHeavyHitters
    extends FunctionUnit1[TwitterRuntime, t.SketchMap, List[(String, Long)]] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def evaluate(
    context: Context[TwitterRuntime],
    sketchMap: t.SketchMap
  ): List[(String, Long)] = {
    val sm = SketchMapAggregator.injection.invert(sketchMap).get
    SketchMapAggregator.monoid.heavyHitters(sm)
  }

  override def description: String = "Gets heavy hitters in SketchMap"

  override def arguments: Seq[String] = Seq(
    "SketchMap thrift"
  )
}

object SketchMapCreate extends FunctionUnit0V[TwitterRuntime, (String, Long), t.SketchMap] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def description: String = "Create a SketchMap"

  override def arguments: Seq[String] = Seq(
    "SketchMap thrift"
  )

  override def evaluate(
    context: Context[TwitterRuntime],
    vargs: Seq[(String, Long)]
  ): t.SketchMap = {
    val sm = SketchMapAggregator.monoid.create(vargs)
    SketchMapAggregator.injection.apply(sm)
  }
}
