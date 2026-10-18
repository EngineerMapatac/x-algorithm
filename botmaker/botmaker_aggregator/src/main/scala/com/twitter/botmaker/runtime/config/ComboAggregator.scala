package com.twitter.botmaker.runtime.config

import com.twitter.botmaker.compiler.tuples.Pair
import com.twitter.botmaker.compiler.tuples.Tuple3
import com.twitter.botmaker.runtime.thriftjava._
import com.twitter.botmaker.compiler.tuples.Tuple

class ComboAggregator2[I1, I2, S1, S2, O1, O2](
  val agg1: AggregatorUnit[I1, S1, O1],
  val agg2: AggregatorUnit[I2, S2, O2])
    extends AggregatorUnit[Pair[I1, I2], Pair[S1, S2], Pair[O1, O2]] {

  override def valueType(config: LocalAggregatorConfig): String = {
    val vt1 = agg1.valueType(config)
    val vt2 = agg2.valueType(config)
    s"Pair<$vt1,$vt2>"
  }

  override def resultType(config: LocalAggregatorConfig): String = {
    val rt1 = agg1.resultType(config)
    val rt2 = agg2.resultType(config)
    s"Pair<$rt1,$rt2>"
  }

  override def description: String = s"ComboAggregator(${agg1.funcName}, ${agg2.funcName})"

  override def zero(config: LocalAggregatorConfig): Pair[S1, S2] = {
    Pair.of(agg1.zero(config), agg2.zero(config))
  }

  override def plus(config: LocalAggregatorConfig, stat: Pair[S1, S2], e: Pair[I1, I2]): Unit = {
    agg1.plus(config, stat.getFirst, e.getFirst)
    agg2.plus(config, stat.getSecond, e.getSecond)
  }

  override def flush(config: LocalAggregatorConfig, stat: Pair[S1, S2]): Pair[O1, O2] = {
    Pair.of(
      agg1.flush(config, stat.getFirst),
      agg2.flush(config, stat.getSecond)
    )
  }

  override def filter(config: LocalAggregatorConfig, stat: Pair[S1, S2]): Boolean = {
    agg1.filter(config, stat.getFirst) || agg2.filter(config, stat.getSecond)
  }
}

class ComboAggregator3[I1, I2, I3, S1, S2, S3, O1, O2, O3](
  val agg1: AggregatorUnit[I1, S1, O1],
  val agg2: AggregatorUnit[I2, S2, O2],
  val agg3: AggregatorUnit[I3, S3, O3])
    extends AggregatorUnit[Tuple3[I1, I2, I3], Tuple3[S1, S2, S3], Tuple3[O1, O2, O3]] {

  override def valueType(config: LocalAggregatorConfig): String = {
    val vt1 = agg1.valueType(config)
    val vt2 = agg2.valueType(config)
    val vt3 = agg3.valueType(config)
    s"Tuple<$vt1,$vt2,$vt3>"
  }

  override def resultType(config: LocalAggregatorConfig): String = {
    val rt1 = agg1.resultType(config)
    val rt2 = agg2.resultType(config)
    val rt3 = agg3.resultType(config)
    s"Tuple<$rt1,$rt2,$rt3>"
  }

  override def description: String =
    s"ComboAggregator(${agg1.funcName}, ${agg2.funcName}, ${agg3.funcName})"

  override def zero(config: LocalAggregatorConfig): Tuple3[S1, S2, S3] = {
    Tuple.of(agg1.zero(config), agg2.zero(config), agg3.zero(config))
  }

  override def plus(
    config: LocalAggregatorConfig,
    stat: Tuple3[S1, S2, S3],
    e: Tuple3[I1, I2, I3]
  ): Unit = {
    agg1.plus(config, stat.getFirst, e.getFirst)
    agg2.plus(config, stat.getSecond, e.getSecond)
    agg3.plus(config, stat.getThird, e.getThird)
  }

  override def flush(
    config: LocalAggregatorConfig,
    stat: Tuple3[S1, S2, S3]
  ): Tuple3[O1, O2, O3] = {
    Tuple.of(
      agg1.flush(config, stat.getFirst),
      agg2.flush(config, stat.getSecond),
      agg3.flush(config, stat.getThird)
    )
  }

  override def filter(config: LocalAggregatorConfig, stat: Tuple3[S1, S2, S3]): Boolean = {
    agg1.filter(config, stat.getFirst) ||
    agg2.filter(config, stat.getSecond) ||
    agg3.filter(config, stat.getThird)
  }
}
