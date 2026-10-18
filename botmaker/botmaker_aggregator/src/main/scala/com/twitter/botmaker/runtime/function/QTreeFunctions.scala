package com.twitter.botmaker.runtime.function

import com.twitter.algebird.QTree
import com.twitter.algebird_internal.{thriftscala => t}
import com.twitter.botmaker._
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.runtime.config.QTreeAggregator

object QTreeQuantile extends FunctionUnit2[TwitterRuntime, t.QTree, Double, Double] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def evaluate(context: Context[TwitterRuntime], arg1: t.QTree, arg2: Double): Double = {
    val qTree = QTreeAggregator.injection.invert(arg1).get
    quantileMean(qTree, arg2)
  }

  override def description: String = "Get percentile from a Qtree"

  override def arguments: Seq[String] = Seq(
    "Qtree thrift",
    "quantile"
  )

  def quantileMean(qTree: QTree[Double], quantile: Double): Double = {
    val count: Long = qTree.count
    val unitQuantile: Double = 1.0d / Math.max(1, count - 1)
    val rawLowerRank: Double = Math.floor(quantile / unitQuantile)
    val halfRatio: Double = (quantile / unitQuantile - rawLowerRank) * 0.5d
    val lowerRank: Double = Math.max(0, Math.min(rawLowerRank, count))
    val upperRank: Double = Math.max(0, Math.min(rawLowerRank + 1, count))
    val lowerBounds = qTree.quantileBounds(lowerRank / count)
    val upperBounds = qTree.quantileBounds(upperRank / count)
    val twiceLowerBound: Double = 0.0d + lowerBounds._1.asInstanceOf[Number].doubleValue +
      lowerBounds._2.asInstanceOf[Number].doubleValue
    val twiceUpperBound: Double = 0.0d + upperBounds._1.asInstanceOf[Number].doubleValue +
      upperBounds._2.asInstanceOf[Number].doubleValue
    twiceLowerBound * (0.5d - halfRatio) + twiceUpperBound * halfRatio
  }
}

object QTreeMerge extends FunctionUnit1V[TwitterRuntime, t.QTree, t.QTree, t.QTree] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global
  override def downstreams: Set[DownstreamService] = Set.empty
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION

  override def description: String = "Merge QTrees"

  override def arguments: Seq[String] = Seq(
    "Qtree thrift to be merged",
    "Qtree thrifts"
  )

  override def evaluate(
    context: Context[TwitterRuntime],
    arg1: t.QTree,
    vargs: Seq[t.QTree]
  ): t.QTree = {
    QTreeAggregator.injection.apply(
      vargs.foldLeft(QTreeAggregator.injection.invert(arg1).get) { (x, y) =>
        x.merge(QTreeAggregator.injection.invert(y).get)
      } compress (QTreeAggregator.compression)
    )
  }
}
