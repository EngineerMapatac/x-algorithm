package com.twitter.botmaker.app.scarecrow.sys

import java.lang.{Double => JDouble}
import java.lang.{Long => JLong}
import java.util.{HashMap => JHashMap}
import java.util.{List => JList}
import java.util.{Map => JMap}
import scala.collection.JavaConverters._
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.BotMakerLog
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit1
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.relevance.feature_store.thrift.FeatureData
import com.twitter.relevance.feature_store.thrift.FeatureValue

object FixBotMakerLog extends FunctionUnit1[ScarecrowRuntime, BotMakerLog, BotMakerLog] {
  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global

  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION

  override def downstreams: Set[DownstreamService] = Set.empty

  override def description: String =
    "Make sure all FeatureValues in the BotMakerLog thrift are correctly typed."

  override def arguments: Seq[String] =
    Seq(
      "a BotMakerLog thrift"
    )

  override def examples: Seq[String] = Nil

  override def evaluate(context: Context[ScarecrowRuntime], log: BotMakerLog): BotMakerLog = {
    log.setDerivedFeatures(fixFeatureMap(log.derivedFeatures))
  }

  private def fixFeatureMap(features: JMap[String, FeatureData]): JMap[String, FeatureData] = {
    if (features == null) {
      features
    } else {
      val returnMap = new JHashMap[String, FeatureData](features.size)
      features.forEach((fn: String, fd: FeatureData) =>
        if (fd != null && fd.featureValue != null) {
          returnMap.put(fn, new FeatureData().setFeatureValue(fixFeatureValue(fd.featureValue)));
        } else {
          returnMap.put(fn, fd)
        })
      returnMap
    }
  }

  private def fixFeatureValue(fv: FeatureValue): FeatureValue = {
    if (fv.isSet(FeatureValue._Fields.STR_MAP_VALUE)) {
      FeatureValue.strMapValue(
        fv.getStrMapValue
          .asInstanceOf[JMap[AnyRef, AnyRef]].asScala.map {
            case (k, v) => stringValue(k) -> stringValue(v)
          }.asJava
      )
    } else if (fv.isSet(FeatureValue._Fields.DOUBLE_MAP_VALUE)) {
      FeatureValue.doubleMapValue(
        fv.getDoubleMapValue
          .asInstanceOf[JMap[AnyRef, JDouble]].asScala.map {
            case (k, v) => stringValue(k) -> v
          }.asJava
      )
    } else if (fv.isSet(FeatureValue._Fields.LONG_MAP_VALUE)) {
      FeatureValue.longMapValue(
        fv.getLongMapValue
          .asInstanceOf[JMap[AnyRef, JLong]].asScala.map {
            case (k, v) => stringValue(k) -> v
          }.asJava
      )
    } else if (fv.isSet(FeatureValue._Fields.STR_LIST_MAP_VALUE)) {
      FeatureValue.strListMapValue(
        fv.getStrListMapValue
          .asInstanceOf[JMap[AnyRef, JList[AnyRef]]].asScala.map {
            case (k, v) => stringValue(k) -> v.asScala.map(stringValue).asJava
          }.asJava
      )
    } else {
      fv
    }
  }

  private def stringValue(x: AnyRef): String = {
    if (x == null) {
      null
    } else {
      x.toString
    }
  }
}
