package com.twitter.botmaker.rule

import com.google.common.collect.ImmutableList
import com.google.common.collect.Maps
import com.google.common.collect.Sets
import com.twitter.botmaker.ResponseCode
import com.twitter.botmaker.RuleResponse
import com.twitter.botmaker.common.FeatureDataConverter
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.compiler.DerivedFeature
import com.twitter.botmaker.compiler.exceptions.FunctionFailure
import com.twitter.botmaker.compiler.exceptions.MissingFeatureException
import com.twitter.botmaker.compiler.tuples.StackFrame
import com.twitter.botmaker.compiler.tuples.StructTuple
import com.twitter.botmaker.compiler.types.StructTupleType
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.feature.OutputFeature
import com.twitter.botmaker.function.misc.DebugOutput
import com.twitter.botmaker.rule.function.ResponseCodes
import com.twitter.relevance.feature_store.thrift.FeatureData
import java.util.{Arrays => JArrays}
import java.util.{List => JList}
import java.util.{Map => JMap}
import java.util.{Set => JSet}
import scala.collection.JavaConverters._

class EventOutputCollector extends OutputCollector with InputProvider {

  override def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ) = ()

  override def isDefinedAt(key: String): Boolean = false

  override def apply(key: String): AnyRef = {
    throw new MissingFeatureException(key)
  }
}

object EventOutputCollector {
  val empty: EventOutputCollector = new EventOutputCollector
}

object EventOutput {

  override def toString: String = "EventOutput"

  def addInputFeature[E](builder: CompilerBuilder[E], systemEvent: String): CompilerBuilder[E] = {
    builder
      .addInputFeature(s"${systemEvent}.outputFeatures", Type.mapOf(Type.STRING, Type.OBJECT))
      .addInputFeature(s"${systemEvent}.debugOutputs", Type.mapOf(Type.STRING, Type.OBJECT))
  }
}

trait EventOutput extends EventOutputCollector {

  val outputFeatures: JMap[String, AnyRef] = Maps.newConcurrentMap()
  val debugOutputs: JMap[String, AnyRef] = Maps.newConcurrentMap()

  override def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ): Unit = {
    namespace match {
      case DebugOutput.NAMESPACE => debugOutputs.put(fn, fv)
      case OutputFeature.NAMESPACE => outputFeatures.put(fn, fv)
      case _ => super.addOutputFeature(namespace, eventType, ruleId, fn, ft, fv)
    }
  }

  override def isDefinedAt(key: String): Boolean = key match {
    case "outputFeatures" => true
    case "debugOutputs" => true
    case _ => super.isDefinedAt(key)
  }

  override def apply(key: String): AnyRef = key match {
    case "outputFeatures" => outputFeatures
    case "debugOutputs" => debugOutputs
    case _ => super.apply(key)
  }
}

object EventOutputFeatureData {

  override def toString: String = "EventOutputFeatureData"

  def addInputFeature[E](builder: CompilerBuilder[E], systemEvent: String): CompilerBuilder[E] = {
    builder
      .addInputFeature(
        s"${systemEvent}.outputFeatureData",
        Type.mapOf(Type.STRING, Type.thriftOf(classOf[FeatureData]))
      )
  }
}

trait EventOutputFeatureData extends EventOutputCollector {

  val outputFeatureData: JMap[String, FeatureData] = Maps.newConcurrentMap()

  override def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ): Unit = {
    namespace match {
      case OutputFeature.NAMESPACE =>
        val fd = FeatureDataConverter.toJavaWithDebugInfo(ft, fv)
        outputFeatureData.put(fn, fd)
      case _ => super.addOutputFeature(namespace, eventType, ruleId, fn, ft, fv)
    }
  }

  override def isDefinedAt(key: String): Boolean = key match {
    case "outputFeatureData" => true
    case _ => super.isDefinedAt(key)
  }

  override def apply(key: String): AnyRef = key match {
    case "outputFeatureData" => outputFeatureData
    case _ => super.apply(key)
  }
}

object RuleResultOutput {
  override def toString: String = "RuleResultOutput"

  def addInputFeature[E](builder: CompilerBuilder[E], systemEvent: String): CompilerBuilder[E] = {
    builder
      .addInputFeature(s"${systemEvent}.violatedRules", Type.setOf(Type.LONG))
      .addInputFeature(s"${systemEvent}.unviolatedRules", Type.setOf(Type.LONG))
      .addInputFeature(s"${systemEvent}.abortedRules", Type.setOf(Type.LONG))
      .addInputFeature(s"${systemEvent}.skippedRules", Type.setOf(Type.LONG))
      .addInputFeature(s"${systemEvent}.timeoutRules", Type.setOf(Type.LONG))
      .addInputFeature(s"${systemEvent}.actionFailedRules", Type.setOf(Type.LONG))
  }
}

trait RuleResultOutput extends EventOutputCollector {

  val violatedRules: JSet[Long] = Sets.newConcurrentHashSet()
  val unviolatedRules: JSet[Long] = Sets.newConcurrentHashSet()
  val abortedRules: JSet[Long] = Sets.newConcurrentHashSet()
  val timeoutRules: JSet[Long] = Sets.newConcurrentHashSet()
  val skippedRules: JSet[Long] = Sets.newConcurrentHashSet()
  val actionFailedRules: JSet[Long] = Sets.newConcurrentHashSet()

  override def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ): Unit = {
    namespace match {
      case RuleResultOutput =>
        fv match {
          case RuleResult.ABORTED | RuleResult.ABORTED_IGNORED =>
            abortedRules.add(ruleId)
          case RuleResult.CANCELLED =>
            timeoutRules.add(ruleId)
          case RuleResult.FAILED | RuleResult.FAILED_IGNORED =>
            violatedRules.add(ruleId)
            actionFailedRules.add(ruleId)
          case RuleResult.SKIPPED =>
            skippedRules.add(ruleId)
          case RuleResult.UNVIOLATED =>
            unviolatedRules.add(ruleId)
          case RuleResult.VIOLATED =>
            violatedRules.add(ruleId)
        }
      case _ => super.addOutputFeature(namespace, eventType, ruleId, fn, ft, fv)
    }
  }

  override def isDefinedAt(key: String): Boolean = key match {
    case "abortedRules" => true
    case "actionFailedRules" => true
    case "skippedRules" => true
    case "timeoutRules" => true
    case "unviolatedRules" => true
    case "violatedRules" => true
    case _ => super.isDefinedAt(key)
  }

  override def apply(key: String): AnyRef = key match {
    case "abortedRules" => abortedRules
    case "actionFailedRules" => actionFailedRules
    case "skippedRules" => skippedRules
    case "timeoutRules" => timeoutRules
    case "violatedRules" => violatedRules
    case "unviolatedRules" => unviolatedRules
    case _ => super.apply(key)
  }
}

object ResponseOutput {
  override def toString: String = "ResponseOutput"

  val ResponseCode: String = "ResponseCode"
  val ExemptionOverride: String = "exemptionOverride"
  val ActionResponse: String = "ActionResponse"

  def addInputFeature[E](builder: CompilerBuilder[E], systemEvent: String): CompilerBuilder[E] = {
    builder
      .addInputFeature(s"${systemEvent}.responseCodes", Type.setOf(Type.STRING))
      .addInputFeature(s"${systemEvent}.finalResponseCode", Type.STRING)
      .addInputFeature(s"${systemEvent}.finalRuleResponse", Type.thriftOf(classOf[RuleResponse]))
      .addInputFeature(s"${systemEvent}.ruleResponseCodes", Type.mapOf(Type.LONG, Type.STRING))
  }
}

trait ResponseOutput extends EventOutputCollector {

  val responseCodeCollection: JMap[Long, ResponseCode] = Maps.newConcurrentMap()
  val exemptionOverrideCollection: JSet[Long] = Sets.newConcurrentHashSet()

  private lazy val ruleResponses: JSet[RuleResponse] = {
    val rs = responseCodeCollection.asScala map {
      case (ruleId, responseCode) =>
        new RuleResponse()
          .setRuleId(ruleId)
          .setCode(responseCode)
          .setShouldOverrideExemption(exemptionOverrideCollection.contains(ruleId))
    }
    rs.toSet.asJava
  }

  def allRuleResponses: JSet[RuleResponse] = ruleResponses

  lazy val finalRuleResponse: RuleResponse =
    ResponseCodes.resolveFinalResponse(allRuleResponses.asScala)
  lazy val (allResponseCodes, allRuleResponseCodes) = {
    val responseCodes: JSet[String] = Sets.newHashSet[String]
    val ruleResponseCodes: JMap[Long, String] = Maps.newHashMap[Long, String]
    allRuleResponses.asScala foreach { t =>
      responseCodes.add(t.code.name)
      if (t.isSetCode && t.code != ResponseCode.NONE) {
        ruleResponseCodes.put(t.ruleId, t.code.name)
      }
    }
    (responseCodes, ruleResponseCodes)
  }

  override def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ): Unit = {
    namespace match {
      case ResponseOutput =>
        fv match {
          case responseCode: ResponseCode if fn == ResponseOutput.ResponseCode =>
            responseCodeCollection.put(ruleId, responseCode)
          case exemptionOverride: java.lang.Boolean
              if fn == ResponseOutput.ExemptionOverride && exemptionOverride =>
            exemptionOverrideCollection.add(ruleId)
          case _ => ()
        }

      case _ => super.addOutputFeature(namespace, eventType, ruleId, fn, ft, fv)
    }
  }

  override def isDefinedAt(key: String): Boolean = key match {
    case "ruleResponseCodes" => true
    case "responseCodes" => true
    case "finalResponseCode" => true
    case "finalRuleResponse" => true
    case _ => super.isDefinedAt(key)
  }

  override def apply(key: String): AnyRef = key match {
    case "ruleResponseCodes" => allRuleResponseCodes
    case "responseCodes" => allResponseCodes
    case "finalResponseCode" => finalRuleResponse.code.name
    case "finalRuleResponse" => finalRuleResponse
    case _ =>
      super.apply(key)
  }
}

object DerivedFeatureOutput {
  override def toString: String = "DerivedFeatureOutput"

  def addInputFeature[E](builder: CompilerBuilder[E], systemEvent: String): CompilerBuilder[E] = {
    builder.addInputFeature(s"${systemEvent}.derivedFeatures", Type.mapOf(Type.STRING, Type.OBJECT))
  }
}

trait DerivedFeatureOutput extends EventOutputCollector {

  val derivedFeatures: JMap[String, AnyRef] = Maps.newConcurrentMap()

  override def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ): Unit = {
    namespace match {
      case _: DerivedFeature => derivedFeatures.put(fn, fv)
      case _ => super.addOutputFeature(namespace, eventType, ruleId, fn, ft, fv)
    }
  }

  override def isDefinedAt(key: String): Boolean = key match {
    case "derivedFeatures" => true
    case _ => super.isDefinedAt(key)
  }

  override def apply(key: String): AnyRef = key match {
    case "derivedFeatures" => derivedFeatures
    case _ => super.apply(key)
  }
}

object DerivedFeatureData {
  override def toString: String = "DerivedFeatureData"

  def addInputFeature[E](builder: CompilerBuilder[E], systemEvent: String): CompilerBuilder[E] = {
    builder
      .addInputFeature(
        s"${systemEvent}.derivedFeatureData",
        Type.mapOf(Type.STRING, Type.thriftOf(classOf[FeatureData]))
      )
  }
}

trait DerivedFeatureData extends EventOutputCollector {

  val derivedFeatureData: JMap[String, FeatureData] = Maps.newConcurrentMap()

  override def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ): Unit = {
    namespace match {
      case _: DerivedFeature =>
        val fd = FeatureDataConverter.toJavaWithDebugInfo(ft, fv)
        derivedFeatureData.put(fn, fd)
      case _ => super.addOutputFeature(namespace, eventType, ruleId, fn, ft, fv)
    }
  }

  override def isDefinedAt(key: String): Boolean = key match {
    case "derivedFeatureData" => true
    case _ => super.isDefinedAt(key)
  }

  override def apply(key: String): AnyRef = key match {
    case "derivedFeatureData" => derivedFeatureData
    case _ => super.apply(key)
  }
}

object FailureOutput extends EventOutputCollector {
  override def toString: String = "FailureOutput"

  def addInputFeature[E](builder: CompilerBuilder[E], systemEvent: String): CompilerBuilder[E] = {
    builder
      .addInputFeature(
        s"${systemEvent}.ruleExceptions",
        Type.mapOf(Type.LONG, tupleType)
      )
  }

  val tupleFieldNames: ImmutableList[String] = ImmutableList.of[String](
    "exception",
    "message",
    "stackTrace"
  )

  val tupleType: StructTupleType = Type.structTupleOf(
    tupleFieldNames,
    ImmutableList.of[Type](
      Type.STRING,
      Type.STRING,
      Type.listOf(StackFrame.TYPE)
    )
  )

  def toTuple(t: Throwable): FailureTuple = {
    t match {
      case ff: FunctionFailure =>
        FailureTuple(
          JArrays.asList[AnyRef](
            ff.getCause.getClass.getName,
            ff.getCause.getMessage,
            ff.getStackFrameMessage
          ))
      case _ =>
        FailureTuple(
          JArrays.asList[AnyRef](
            t.getClass.getName,
            t.getMessage,
            ImmutableList.of[StackFrame]()
          ))
    }
  }

  case class FailureTuple(fieldValues: JList[AnyRef])
      extends StructTuple(tupleFieldNames, fieldValues) {}
}

trait FailureOutput extends EventOutputCollector {

  val ruleExceptions: JMap[Long, FailureOutput.FailureTuple] = Maps.newConcurrentMap()

  override def addOutputFeature(
    namespace: AnyRef,
    eventType: String,
    ruleId: Long,
    fn: String,
    ft: Type,
    fv: AnyRef
  ): Unit = {
    namespace match {
      case FailureOutput =>
        fv match {
          case t: FailureOutput.FailureTuple =>
            ruleExceptions.put(ruleId, t)
          case _ => ()
        }

      case _ =>
        super.addOutputFeature(namespace, eventType, ruleId, fn, ft, fv)
    }
  }

  override def isDefinedAt(key: String): Boolean = key match {
    case "ruleExceptions" => true
    case _ => super.isDefinedAt(key)
  }

  override def apply(key: String): AnyRef = key match {
    case "ruleExceptions" => ruleExceptions
    case _ => super.apply(key)
  }
}
