package com.twitter.botmaker.runtime.config

import com.twitter.algebird.QTree
import com.twitter.algebird_internal.injection.QTreeImplicits
import com.twitter.algebird_internal.injection.ThriftImplicits._
import com.twitter.algebird_internal.{thriftscala => t}
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.runtime.thriftjava._
import com.twitter.bijection.Injection

case class QTreeStats() {

  private var qt: QTree[Double] = null

  def isEmpty: Boolean = qt == null

  def add(e: Double, k: Int): Unit = this.synchronized {
    qt = if (qt == null) {
      QTree(e)
    } else {
      qt.merge(QTree(e)).compress(k)
    }
  }

  def flush(): t.QTree = this.synchronized {
    val result = qt
    qt = null

    QTreeAggregator.injection.apply(result)
  }
}

object QTreeAggregator extends AggregatorUnit[Double, QTreeStats, t.QTree] {

  val compression = 8

  val injection: Injection[QTree[Double], t.QTree] = QTreeImplicits.qtreeToThriftInjection[Double]

  override def description: String = "Qtree Aggregator"

  override def valueType(config: LocalAggregatorConfig): String = {
    Type.DOUBLE.toString
  }
  override def resultType(config: LocalAggregatorConfig): String = {
    Type.thriftStructOf(classOf[t.QTree]).toString
  }

  override def zero(config: LocalAggregatorConfig): QTreeStats = {
    QTreeStats()
  }

  override def plus(config: LocalAggregatorConfig, stat: QTreeStats, e: Double): Unit = {
    stat.add(e, compression)
  }

  override def flush(config: LocalAggregatorConfig, stat: QTreeStats): t.QTree = {
    stat.flush
  }

  override def filter(config: LocalAggregatorConfig, stat: QTreeStats): Boolean = {
    !stat.isEmpty
  }
}
