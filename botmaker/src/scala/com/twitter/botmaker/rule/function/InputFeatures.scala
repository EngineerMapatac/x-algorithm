package com.twitter.botmaker.rule.function

import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit0
import com.twitter.botmaker.rule.RuleContext
import com.twitter.botmaker.runtime.TwitterRuntime

object RuleId extends FunctionUnit0[TwitterRuntime, Long] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Rule
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def evaluate(context: Context[TwitterRuntime]): Long = {
    context.getTrackingId
  }

  override def description: String = "Rule Id; returns -1 if unknown (e.g. in Run Expression)"

  override def arguments: Seq[String] = Nil
}

object ClusterName extends FunctionUnit0[TwitterRuntime, String] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def evaluate(context: Context[TwitterRuntime]): String = {
    context.getRuntime.info.map(_.cluster).getOrElse("unknown")
  }

  override def description: String = "Cluster Name"

  override def arguments: Seq[String] = Nil
}

object ServiceName extends FunctionUnit0[TwitterRuntime, String] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description: String = "Service Name"
  override def arguments: Seq[String] = Nil

  override def evaluate(context: Context[TwitterRuntime]): String = {
    context.getRuntime.info.map(_.serviceName).getOrElse("unknown")
  }
}

object ShardId extends FunctionUnit0[TwitterRuntime, Long] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def evaluate(context: Context[TwitterRuntime]): Long = {
    context.getRuntime.info.map(_.shardId).getOrElse(0).toLong
  }

  override def description: String = "Shard Id"

  override def arguments: Seq[String] = Nil
}

object IsAdminInstance extends FunctionUnit0[TwitterRuntime, Boolean] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def evaluate(context: Context[TwitterRuntime]): Boolean = {
    context.getRuntime.info.map(_.isAdminInstance).getOrElse(false)
  }

  override def description: String = "If the running instance is an administrative instance"

  override def arguments: Seq[String] = Nil
}

object EventType extends FunctionUnit0[TwitterRuntime, String] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def evaluate(context: Context[TwitterRuntime]): String = {
    context.getRootContext match {
      case ruleContext: RuleContext[_] => ruleContext.eventType
      case _ => ""
    }
  }

  override def description: String = "Event Type"

  override def arguments: Seq[String] = Nil
}

object EventTime extends FunctionUnit0[TwitterRuntime, Long] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  override def evaluate(context: Context[TwitterRuntime]): Long = {
    context.startMillis
  }

  override def description: String = "Event Time"

  override def arguments: Seq[String] = Nil
}
