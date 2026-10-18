package com.twitter.botmaker.runtime.config

import com.twitter.algebird_internal.{thriftscala => t}
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.tuples.Pair
import com.twitter.botmaker.runtime.thriftjava._
import com.twitter.algebird.SketchMap
import com.twitter.algebird.SketchMapParams
import com.twitter.algebird_internal.injection.SketchMapInjection
import com.twitter.algebird_internal.injection.ThriftImplicits
import com.twitter.algebird.SketchMapMonoid

case class SketchMapStats() {

  private var sm = SketchMapAggregator.monoid.zero

  def isEmpty: Boolean = sm == SketchMapAggregator.monoid.zero

  def add(b: Pair[String, Long]): Unit = this.synchronized {
    sm = SketchMapAggregator.monoid.plus(
      sm,
      SketchMapAggregator.monoid.create(b.getFirst -> b.getSecond)
    )
  }

  def flush(): t.SketchMap = this.synchronized {
    val result = sm
    sm = SketchMapAggregator.monoid.zero

    SketchMapAggregator.injection.apply(result)
  }
}

object SketchMapAggregator extends AggregatorUnit[Pair[String, Long], SketchMapStats, t.SketchMap] {

  import ThriftImplicits._

  val DELTA = 1e-8

  val EPS = 0.001

  val SEED = 1

  val HEAVY_HITTERS_COUNT = 10

  val PARAMS: SketchMapParams[String] =
    SketchMapParams[String](SEED, EPS, DELTA, HEAVY_HITTERS_COUNT) {
      case s: String => s.toCharArray.map(_.toByte)
    }

  val monoid: SketchMapMonoid[String, Long] = SketchMap.monoid[String, Long](PARAMS)

  val injection: SketchMapInjection[String, Long] = new SketchMapInjection(monoid)

  override def description: String = "SketchMap Aggregator"

  override def valueType(config: LocalAggregatorConfig): String = {
    Type.pairOf(Type.STRING, Type.LONG).toString
  }
  override def resultType(config: LocalAggregatorConfig): String = {
    Type.thriftStructOf(classOf[t.SketchMap]).toString
  }

  override def zero(config: LocalAggregatorConfig): SketchMapStats = {
    SketchMapStats()
  }

  override def plus(
    config: LocalAggregatorConfig,
    stat: SketchMapStats,
    b: Pair[String, Long]
  ): Unit = {
    stat.add(b)
  }

  override def flush(config: LocalAggregatorConfig, stat: SketchMapStats): t.SketchMap = {
    stat.flush
  }

  override def filter(config: LocalAggregatorConfig, stat: SketchMapStats): Boolean = {
    !stat.isEmpty
  }
}
