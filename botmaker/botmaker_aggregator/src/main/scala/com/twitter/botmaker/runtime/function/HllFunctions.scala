package com.twitter.botmaker.runtime.function

import com.twitter.algebird_internal.{thriftscala => t}
import com.twitter.botmaker._
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.runtime.config.HllAggregator

object HllEstimatedSize extends FunctionUnit1[TwitterRuntime, t.Hll, Double] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def evaluate(context: Context[TwitterRuntime], arg1: t.Hll): Double = {
    val hll = HllAggregator.injection.invert(arg1)
    hll.estimatedSize
  }

  override def description: String = "Get estimated Hll size"

  override def arguments: Seq[String] = Seq(
    "Hll thrift"
  )
}

object HllMerge extends FunctionUnit0V[TwitterRuntime, t.Hll, t.Hll] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def description: String = "Merge Hlls"

  override def arguments: Seq[String] = Seq(
    "Hll thrift"
  )

  override def evaluate(context: Context[TwitterRuntime], vargs: Seq[t.Hll]): t.Hll = {
    HllAggregator.injection.apply(
      vargs.foldLeft(HllAggregator.monoid.zero) { (x, y) =>
        HllAggregator.monoid.plus(x, HllAggregator.injection.invert(y))
      }
    )
  }
}
