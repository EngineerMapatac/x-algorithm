package com.twitter.botmaker.runtime.config

import com.twitter.algebird.HLL
import com.twitter.algebird_internal.injection.HyperLogLogImplicits
import com.twitter.algebird_internal.{thriftscala => t}
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.runtime.thriftjava._
import com.twitter.algebird.HyperLogLogMonoid
import java.nio.ByteBuffer
import com.twitter.bijection.Bijection

case class HllStats() {

  private var hll: HLL = null

  def isEmpty: Boolean = hll == null

  def add(b: Array[Byte]): Unit = this.synchronized {
    hll = if (hll == null) {
      HllAggregator.monoid.create(b)
    } else {
      HllAggregator.monoid.plus(hll, HllAggregator.monoid.create(b))
    }
  }

  def flush(): t.Hll = this.synchronized {
    val result = hll
    hll = null

    HllAggregator.injection.apply(result)
  }
}

object HllAggregator extends AggregatorUnit[ByteBuffer, HllStats, t.Hll] {

  val monoid: HyperLogLogMonoid = new HyperLogLogMonoid(9)

  val injection: Bijection[HLL, t.Hll] = HyperLogLogImplicits.hllToThrift$

  override def description: String = "Hll Aggregator"

  override def valueType(config: LocalAggregatorConfig): String = {
    Type.BINARY.toString
  }
  override def resultType(config: LocalAggregatorConfig): String = {
    Type.thriftStructOf(classOf[t.Hll]).toString
  }

  override def zero(config: LocalAggregatorConfig): HllStats = {
    HllStats()
  }

  override def plus(config: LocalAggregatorConfig, stat: HllStats, b: ByteBuffer): Unit = {
    stat.add(b.array)
  }

  override def flush(config: LocalAggregatorConfig, stat: HllStats): t.Hll = {
    stat.flush
  }

  override def filter(config: LocalAggregatorConfig, stat: HllStats): Boolean = {
    !stat.isEmpty
  }
}
