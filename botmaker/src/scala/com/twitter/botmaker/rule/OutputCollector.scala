package com.twitter.botmaker.rule

import com.twitter.botmaker.compiler.types.Type

trait OutputCollector {
  def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ): Unit
}

object OutputCollector {
  val empty: OutputCollector = new OutputCollector {
    override def addOutputFeature(
      namespace: AnyRef,
      eventType: String,
      ruleId: Long,
      fn: String,
      ft: Type,
      fv: AnyRef
    ): Unit = ()
  }
}
