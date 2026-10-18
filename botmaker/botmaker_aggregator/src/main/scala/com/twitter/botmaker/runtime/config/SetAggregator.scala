package com.twitter.botmaker.runtime.config

import com.google.common.collect.Sets
import com.twitter.botmaker.runtime.thriftjava._
import java.{util => ju}

case class SetStats() {

  private var set = Sets.newConcurrentHashSet[AnyRef]()

  def size: Int = set.size

  def add(e: AnyRef): Boolean = {
    set.add(e)
  }

  def flush(): ju.Set[AnyRef] = {
    val result = set
    set = Sets.newConcurrentHashSet[AnyRef]()
    result
  }
}

object SetAggregator extends AggregatorUnit[AnyRef, SetStats, java.util.Set[AnyRef]] {

  override def valueType(config: LocalAggregatorConfig): String = {
    config.valueType
  }
  override def resultType(config: LocalAggregatorConfig): String = {
    s"Set<${config.valueType}>"
  }

  override def description: String = "Set Aggregator"

  override def zero(config: LocalAggregatorConfig): SetStats = {
    SetStats()
  }

  override def plus(config: LocalAggregatorConfig, stat: SetStats, e: AnyRef): Unit = {
    stat.add(e)
  }

  override def flush(config: LocalAggregatorConfig, stat: SetStats): java.util.Set[AnyRef] = {
    stat.flush
  }

  override def filter(config: LocalAggregatorConfig, stat: SetStats): Boolean = {
    stat.size > 0
  }
}
