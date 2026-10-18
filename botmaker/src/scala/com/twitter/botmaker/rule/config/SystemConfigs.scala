package com.twitter.botmaker.rule.config

import com.google.common.collect.ImmutableList
import com.google.common.collect.Maps
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.common.FeatureDataConverter
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerFunction
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.FunctionNode2
import com.twitter.botmaker.function.FunctionNode3
import com.twitter.botmaker.runtime.config._
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.util.Duration

import java.util

class SystemConfigs
    extends ServiceConfigs
    with RegexOptionConfigs
    with EventTypeConfigs
    with TeamConfigs {

  val eventMappings: util.Map[String, util.List[String]] = Maps.newConcurrentMap()
  val eventTimeouts: util.Map[String, Duration] = Maps.newConcurrentMap()
  val inputFeatureTypes: util.Map[String, Type] = Maps.newConcurrentMap()
}

object EventMapping {
  val signature: ASTNode.Signature = new ASTNode.Signature(
    ImmutableList.of(Type.STRING, Type.listOf(Type.STRING)),
    Type.UNIT
  )
}

@BotMakerFunction(
  argTypes = Array("String", "List<String>"),
  arguments = Array("event type", "a list of event types this event will be mapped to"),
  returnType = "Unit",
  description = "config an event mapping.",
  actionLevel = ActionLevel.NO_ACTION
)
class EventMapping(exprText: String, children: ImmutableList[ASTNode[_]])
    extends FunctionNode2[SystemConfigs, String, util.List[String]](exprText, children) {

  override protected def getCacheLevel = CacheLevel.Never
  override def getSignature: Signature = EventMapping.signature
  override protected def apply(
    context: Context[SystemConfigs],
    eventType: String,
    mappings: util.List[String]
  ): AnyRef = {
    context.getRuntime.eventMappings.put(eventType, mappings)
    unit
  }
}

object EventTimeout {
  val signature: ASTNode.Signature = new ASTNode.Signature(
    ImmutableList.of(Type.STRING, Type.LONG),
    Type.UNIT
  )
}

@BotMakerFunction(
  argTypes = Array("String", "Long"),
  arguments = Array("event type", "timeout in milliseconds"),
  returnType = "Unit",
  description = "config an event timeout.",
  actionLevel = ActionLevel.NO_ACTION
)
class EventTimeout(exprText: String, children: ImmutableList[ASTNode[_]])
    extends FunctionNode2[SystemConfigs, String, Long](exprText, children) {

  override protected def getCacheLevel = CacheLevel.Never

  override def getSignature: Signature = EventTimeout.signature

  override protected def apply(
    context: Context[SystemConfigs],
    eventType: String,
    timeout: Long
  ): AnyRef = {
    val duration = Duration.fromMilliseconds(timeout)
    context.getRuntime.eventTimeouts.put(eventType, duration)
    unit
  }
}

object InputFeatureType {
  val signature: ASTNode.Signature = new ASTNode.Signature(
    ImmutableList.of(Type.STRING, Type.STRING, Type.STRING),
    Type.UNIT
  )
}

@BotMakerFunction(
  argTypes = Array("String", "String", "String"),
  arguments = Array("input feature namespace", "input feature name", "input feature type name"),
  returnType = "Unit",
  description = "config inputfeature name type mapping.",
  actionLevel = ActionLevel.NO_ACTION
)
class InputFeatureType(exprText: String, children: ImmutableList[ASTNode[_]])
    extends FunctionNode3[SystemConfigs, String, String, String](exprText, children) {

  override protected def getCacheLevel = CacheLevel.Never

  override def getSignature: Signature = InputFeatureType.signature

  override protected def apply(
    context: Context[SystemConfigs],
    featureNameSpace: String,
    featureName: String,
    featureTypeName: String
  ): AnyRef = {

    val fullname = if (featureNameSpace.isEmpty) {
      featureName
    } else {
      String.format("%s.%s", featureNameSpace, featureName);
    }

    context.getRuntime.inputFeatureTypes.put(
      fullname,
      FeatureDataConverter.getBotMakerType(featureTypeName)
    )

    unit
  }
}
