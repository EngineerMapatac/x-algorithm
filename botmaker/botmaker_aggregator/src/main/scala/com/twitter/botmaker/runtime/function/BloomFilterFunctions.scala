package com.twitter.botmaker.runtime.function

import com.twitter.algebird_internal.{thriftscala => t}
import com.twitter.botmaker._
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.runtime.config.BloomFilterAggregator

object BloomFilterContains extends FunctionUnit2[TwitterRuntime, t.BloomFilter, String, Boolean] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def evaluate(
    context: Context[TwitterRuntime],
    bloomFilter: t.BloomFilter,
    item: String
  ): Boolean = {
    val bf = BloomFilterAggregator.injection.invert(bloomFilter)
    bf.get.maybeContains(item)
  }

  override def description: String = "Bloom filter may contains an item"

  override def arguments: Seq[String] = Seq(
    "BloomFilter thrift"
  )
}

object BloomFilterCreate extends FunctionUnit0V[TwitterRuntime, String, t.BloomFilter] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def description: String = "Create a BloomFilter"

  override def arguments: Seq[String] = Seq(
    "BloomFilter thrift"
  )

  override def evaluate(context: Context[TwitterRuntime], vargs: Seq[String]): t.BloomFilter = {
    val bf = BloomFilterAggregator.monoid.create(vargs: _*)
    BloomFilterAggregator.injection.apply(bf)
  }
}
