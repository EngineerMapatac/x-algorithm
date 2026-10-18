package com.twitter.botmaker.runtime.config

import com.twitter.algebird_internal.{thriftscala => t}
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.runtime.thriftjava._
import com.twitter.algebird.BF
import com.twitter.algebird.BloomFilter
import com.twitter.algebird_internal.injection.BloomFilterToThriftInjection
import com.twitter.algebird.BloomFilterMonoid

case class BloomFilterStats() {

  private var bf: BF[String] = null

  def isEmpty: Boolean = bf == null

  def add(b: String): Unit = this.synchronized {
    bf = if (bf == null) {
      BloomFilterAggregator.monoid.create(b)
    } else {
      bf + b
    }
  }

  def flush(): t.BloomFilter = this.synchronized {
    val result = bf
    bf = null

    BloomFilterAggregator.injection.apply(result)
  }
}

object BloomFilterAggregator extends AggregatorUnit[String, BloomFilterStats, t.BloomFilter] {

  val n = 1000000

  val fp = 0.001

  val monoid: BloomFilterMonoid[String] = BloomFilter[String](n, fp)

  val injection: BloomFilterToThriftInjection = new BloomFilterToThriftInjection()(monoid)

  override def description: String = "BloomFilter Aggregator"

  override def valueType(config: LocalAggregatorConfig): String = {
    Type.STRING.toString
  }
  override def resultType(config: LocalAggregatorConfig): String = {
    Type.thriftStructOf(classOf[t.BloomFilter]).toString
  }

  override def zero(config: LocalAggregatorConfig): BloomFilterStats = {
    BloomFilterStats()
  }

  override def plus(config: LocalAggregatorConfig, stat: BloomFilterStats, b: String): Unit = {
    stat.add(b)
  }

  override def flush(config: LocalAggregatorConfig, stat: BloomFilterStats): t.BloomFilter = {
    stat.flush
  }

  override def filter(config: LocalAggregatorConfig, stat: BloomFilterStats): Boolean = {
    !stat.isEmpty
  }
}
