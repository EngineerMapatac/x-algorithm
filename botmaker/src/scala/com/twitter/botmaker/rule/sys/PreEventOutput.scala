package com.twitter.botmaker.rule.sys

import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.rule.OutputCollector
import com.twitter.botmaker.runtime.Runtime
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit0
import com.twitter.botmaker.FunctionUnit1
import com.twitter.util.Duration

trait PreEventOutput {

  def skipEvent: Boolean
  def skipRules: Set[Long]
  def eventTimeout: Duration
}

object ProfilingOption {
  val none = "NONE"
  val downstream = "DOWNSTREAM"
  val funccall = "FUNCCALL"
  val execinfo = "EXECINFO"
  val exprcache = "EXPRCACHE"

  override def toString: String = "ProfilingOption"
}

object PreEventOutput {

  val empty: PreEventOutput = new PreEventOutput {
    def skipEvent: Boolean = false
    def eventTimeout: Duration = Duration.Top
    def skipRules: Set[Long] = Set.empty
  }
}

class PreEventOutputCollector extends OutputCollector with PreEventOutput {

  var skipEvent: Boolean = false
  var eventTimeout: Duration = Duration.Top
  var skipRules: Set[Long] = Set.empty

  override def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ): Unit = namespace match {

    case SkipEvent =>
      skipEvent = true

    case EventTimeout =>
      fv match {
        case t: java.lang.Long =>
          eventTimeout = Duration.fromMilliseconds(t)
      }

    case SkipRule =>
      fv match {
        case rule: java.lang.Long =>
          skipRules = skipRules + rule
      }

    case _ => ()
  }
}

object EventTimeout extends FunctionUnit1[Runtime, Long, Unit] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def description: String =
    "Specify event timeout for current event in preprocessing rules."

  override def arguments: Seq[String] = Seq("event timeout in milliseconds")

  override def evaluate(context: Context[Runtime], timeout: Long): Unit = {
    context.addOutputFeature(EventTimeout, "EventTimeout", Type.LONG, timeout)
  }
}

object SkipRule extends FunctionUnit1[Runtime, Long, Unit] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def description: String =
    """Skip the given rule for current event in preprocessing rules.
      For example, if the rule has been rate limited""""

  override def arguments: Seq[String] = Seq("Rule Id")

  override def evaluate(context: Context[Runtime], ruleId: Long): Unit = {
    context.addOutputFeature(SkipRule, "SkipRule", Type.LONG, ruleId)
  }
}

object SkipEvent extends FunctionUnit0[Runtime, Unit] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description: String = "Skip all the rules for current event in preprocessing rules."

  override def arguments: Seq[String] = Nil

  override def evaluate(context: Context[Runtime]): Unit = {
    context.addOutputFeature(SkipEvent, "SkipEvent", Type.BOOLEAN, true)
  }
}
