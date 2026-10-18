package com.twitter.botmaker.rule

object RuleResult extends Enumeration {
  type RuleResult = Value
  val NONE, VIOLATED, UNVIOLATED, ABORTED, ABORTED_IGNORED, FAILED, FAILED_IGNORED, CANCELLED,
    SKIPPED = Value
}
