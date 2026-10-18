package com.twitter.botmaker.rule

trait StatCollector {
  def reportStat(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    statName: String,
    value: Long
  ): Unit
}

object StatCollector {
  val empty: StatCollector = new StatCollector {
    override def reportStat(
      namespace: AnyRef,
      eventType: String,
      ruleId: Long,
      statName: String,
      value: Long
    ): Unit = ()
  }
}
