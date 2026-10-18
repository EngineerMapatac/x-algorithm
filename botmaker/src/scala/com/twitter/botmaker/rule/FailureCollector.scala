package com.twitter.botmaker.rule

trait FailureCollector {
  def reportFailure(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    category: String,
    t: Throwable
  ): Unit
}

object FailureCollector {
  val empty: FailureCollector = new FailureCollector {
    override def reportFailure(
      namespace: AnyRef,
      eventType: String,
      ruleId: Long,
      category: String,
      exception: Throwable
    ): Unit = ()
  }
}
