package com.twitter.botmaker.runtime.config

import com.twitter.botmaker.compiler.tuples
import com.twitter.botmaker.compiler.tuples.Tuple
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.runtime.thriftjava._

case class SimpleStats() {

  var count: Long = 0L
  var sum: Long = 0L
  var max: Long = Long.MinValue
  var min: Long = Long.MaxValue

  def add(amount: Long): Unit = this.synchronized {
    count += 1
    sum += amount
    max = Math.max(max, amount)
    min = Math.min(min, amount)
  }

  def flush(): tuples.Tuple4[Long, Long, Long, Long] =
    this.synchronized {
      val result = Tuple.of(count, sum, max, min)
      count = 0
      sum = 0
      max = 0
      min = 0
      result
    }
}

object SimpleStatsAggregator extends AggregatorUnit[Long, SimpleStats, Tuple] {

  override def valueType(config: LocalAggregatorConfig): String = {
    Type.LONG.toString
  }

  override def resultType(config: LocalAggregatorConfig): String = {
    Type.tupleOf(Type.LONG, Type.LONG, Type.LONG, Type.LONG).toString
  }

  override def description: String = "Basic Stats Aggregator"

  override def zero(config: LocalAggregatorConfig): SimpleStats = {
    SimpleStats()
  }

  override def plus(config: LocalAggregatorConfig, stat: SimpleStats, amount: Long): Unit = {
    stat.add(amount)
  }

  override def flush(config: LocalAggregatorConfig, stat: SimpleStats): Tuple = {
    stat.flush
  }

  override def filter(config: LocalAggregatorConfig, stat: SimpleStats): Boolean = {
    stat.count > 0
  }
}
