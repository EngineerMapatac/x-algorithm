package com.twitter.botmaker.runtime.config

import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.runtime.thriftjava.LocalAggregatorConfig
import java.util.concurrent.atomic.AtomicLong

object CounterAggregator extends AggregatorUnit[Long, AtomicLong, Long] {

  override def valueType(config: LocalAggregatorConfig): String = {
    Type.LONG.toString
  }

  override def resultType(config: LocalAggregatorConfig): String = {
    Type.LONG.toString
  }

  override def zero(config: LocalAggregatorConfig): AtomicLong = {
    new AtomicLong(0L)
  }

  override def plus(config: LocalAggregatorConfig, stat: AtomicLong, amount: Long): Unit = {
    stat.addAndGet(amount)
  }

  override def flush(config: LocalAggregatorConfig, stat: AtomicLong): Long = {
    stat.getAndSet(0L)
  }

  override def description: String = "Counter Aggregator"

  override def filter(config: LocalAggregatorConfig, stat: AtomicLong): Boolean = {
    stat.get > 0
  }
}
