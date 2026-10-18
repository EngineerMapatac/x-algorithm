package com.twitter.botmaker.rule

import com.google.common.collect.ImmutableMap
import java.util.{Map => JMap}
import scala.collection.JavaConverters._
import com.twitter.botmaker.common.FeatureDataConverter
import com.twitter.botmaker.compiler.exceptions.MissingFeatureException
import com.twitter.relevance.feature_store.thrift.{FeatureData => JavaFeatureData}
import com.twitter.relevance.feature_store.thriftscala.{FeatureData => ScalaFeatureData}

object InputProvider {

  val INPUT_FEATURES = "INPUT_FEATURES"

  val INPUT_FEATURE_DATA = "INPUT_FEATURE_DATA"

  val empty: PartialFunction[Any, Nothing] = PartialFunction.empty

  def apply(inputFeatures: JMap[String, AnyRef]): InputProvider = {
    new InputProvider {
      override def isDefinedAt(key: String): Boolean = key match {
        case INPUT_FEATURES => true
        case _ if inputFeatures.containsKey(key) => true
        case _ => false
      }

      override def apply(key: String): AnyRef = key match {
        case INPUT_FEATURES => inputFeatures
        case _ if inputFeatures.containsKey(key) => inputFeatures.get(key)
        case _ => throw new MissingFeatureException(key)
      }
    }
  }

  def transform(inputFeatureData: JMap[String, JavaFeatureData]): InputProvider = {
    new InputProvider {

      private[this] val inputFeatures: ImmutableMap[String, AnyRef] = {
        val builder = ImmutableMap.builder[String, AnyRef]()

        inputFeatureData.asScala foreach {
          case (k, v) =>
            if (v.isSetFeatureValue && v.featureValue.isSet) {
              builder.put(k, v.featureValue.getFieldValue)
            }
        }
        builder.build()
      }

      override def isDefinedAt(key: String): Boolean = key match {
        case INPUT_FEATURES => true
        case INPUT_FEATURE_DATA => true
        case _ => inputFeatures.containsKey(key)
      }

      override def apply(key: String): AnyRef = key match {
        case INPUT_FEATURES =>
          inputFeatures
        case INPUT_FEATURE_DATA =>
          inputFeatureData
        case _ =>
          inputFeatures.get(key)
      }
    }
  }

  def transform(inputFeatureData: scala.collection.Map[String, ScalaFeatureData]): InputProvider = {
    new InputProvider {

      private[this] val javaInputFeatures = inputFeatureData collect {
        case (k: String, v: ScalaFeatureData) if v.featureValue.isDefined =>
          k -> FeatureDataConverter.featureDataToJavaFeatureData(v)
      }

      private[this] val inputFeatureMap = javaInputFeatures.asJava
      private[this] val inputFeatures = javaInputFeatures
        .mapValues(_.featureValue.getFieldValue).asJava

      override def isDefinedAt(key: String): Boolean = {
        key == INPUT_FEATURES ||
        key == INPUT_FEATURE_DATA ||
        inputFeatures.containsKey(key)
      }

      override def apply(key: String): AnyRef = {
        if (key == INPUT_FEATURES) {
          inputFeatures
        } else if (key == INPUT_FEATURE_DATA) {
          inputFeatureMap
        } else {
          inputFeatures.get(key)
        }
      }
    }
  }
}
