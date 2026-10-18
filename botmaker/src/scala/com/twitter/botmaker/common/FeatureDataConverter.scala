package com.twitter.botmaker.common

import scala.collection.Map
import scala.collection.JavaConverters._
import scala.collection.convert.WrapAsJava
import com.google.common.collect.ImmutableMap
import com.twitter.botmaker.compiler.exceptions.RuntimeFailure
import com.twitter.botmaker.compiler.types.ThriftStructType
import com.twitter.botmaker.compiler.types.ThriftType
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.relevance.feature_store.thrift.{FeatureData => JavaFeatureData}
import com.twitter.relevance.feature_store.thrift.{FeatureValue => JavaFeatureValue}
import com.twitter.relevance.feature_store.thriftscala.FeatureValue._
import com.twitter.relevance.feature_store.thriftscala.{FeatureData => ScalaFeatureData}
import com.twitter.relevance.feature_store.thriftscala.{FeatureValue => ScalaFeatureValue}
import com.twitter.scrooge.ThriftStruct
import com.twitter.spam.botmaker_features.BotMakerFeatures

object FeatureDataConverter {

  private def elementType(collection: java.util.Collection[AnyRef]): Type = {
    if (collection.size == 0) {
      Type.OBJECT
    } else {
      collection.asScala.foldLeft(null.asInstanceOf[Type]) {
        case (null, o) => ofType(o)
        case (t, o) => Type.getClosestCommonSuperType(t, ofType(o))
      }
    }
  }

  private def collectionType(collection: java.util.Collection[AnyRef]): Type = {
    collection match {
      case _: java.util.List[_] => Type.listOf(elementType(collection))
      case _: java.util.Set[_] => Type.setOf(elementType(collection))
      case _ => Type.collectionOf(elementType(collection))
    }
  }

  def ofType(obj: AnyRef): Type = obj match {

    case null => Type.OBJECT
    case _: String => Type.STRING
    case _: java.lang.Long => Type.LONG
    case _: java.lang.Integer => Type.INTEGER
    case _: java.lang.Short => Type.SHORT
    case _: java.lang.Byte => Type.BYTE
    case _: java.lang.Double => Type.DOUBLE
    case _: java.lang.Boolean => Type.BOOLEAN
    case x: TwitterRuntime.Thrift[_, _] => Type.thriftOf(x.getClass)
    case x: ThriftStruct => Type.thriftStructOf(x.getClass)
    case x: java.util.Collection[AnyRef] => collectionType(x)
    case x: java.util.Map[AnyRef, AnyRef] =>
      val keyType = elementType(x.keySet())
      val valueType = elementType(x.values())
      Type.mapOf(keyType, valueType)
    case _ =>
      Type.OBJECT
  }

  val featureValueConvterMap: Map[AnyRef, ScalaFeatureValue => AnyRef] =
    Map[AnyRef, (ScalaFeatureValue) => AnyRef](
      classOf[ShortValue] -> { (obj: ScalaFeatureValue) =>
        Short.box(obj.asInstanceOf[ShortValue].shortValue)
      },
      classOf[IntValue] -> { (obj: ScalaFeatureValue) =>
        Int.box(obj.asInstanceOf[IntValue].intValue)
      },
      classOf[LongValue] -> { (obj: ScalaFeatureValue) =>
        Long.box(obj.asInstanceOf[LongValue].longValue)
      },
      classOf[DoubleValue] -> { (obj: ScalaFeatureValue) =>
        Double.box(obj.asInstanceOf[DoubleValue].doubleValue)
      },
      classOf[BooleanValue] -> { (obj: ScalaFeatureValue) =>
        Boolean.box(obj.asInstanceOf[BooleanValue].booleanValue)
      },
      classOf[StrValue] -> { (obj: ScalaFeatureValue) => obj.asInstanceOf[StrValue].strValue },
      classOf[ShortSetValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.setAsJavaSet(obj.asInstanceOf[ShortSetValue].shortSetValue)
      },
      classOf[IntSetValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.setAsJavaSet(obj.asInstanceOf[IntSetValue].intSetValue)
      },
      classOf[LongSetValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.setAsJavaSet(obj.asInstanceOf[LongSetValue].longSetValue)
      },
      classOf[StrSetValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.setAsJavaSet(obj.asInstanceOf[StrSetValue].strSetValue)
      },
      classOf[ShortListValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.seqAsJavaList(obj.asInstanceOf[ShortListValue].shortListValue)
      },
      classOf[IntListValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.seqAsJavaList(obj.asInstanceOf[IntListValue].intListValue)
      },
      classOf[LongListValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.seqAsJavaList(obj.asInstanceOf[LongListValue].longListValue)
      },
      classOf[DoubleListValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.seqAsJavaList(obj.asInstanceOf[DoubleListValue].doubleListValue)
      },
      classOf[BooleanListValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.seqAsJavaList(obj.asInstanceOf[BooleanListValue].booleanListValue)
      },
      classOf[StrListValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.seqAsJavaList(obj.asInstanceOf[StrListValue].strListValue)
      },
      classOf[ShortMapValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.mapAsJavaMap(obj.asInstanceOf[ShortMapValue].shortMapValue)
      },
      classOf[IntMapValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.mapAsJavaMap(obj.asInstanceOf[IntMapValue].intMapValue)
      },
      classOf[LongMapValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.mapAsJavaMap(obj.asInstanceOf[LongMapValue].longMapValue)
      },
      classOf[DoubleMapValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.mapAsJavaMap(obj.asInstanceOf[DoubleMapValue].doubleMapValue)
      },
      classOf[BooleanMapValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.mapAsJavaMap(obj.asInstanceOf[BooleanMapValue].booleanMapValue)
      },
      classOf[StrMapValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.mapAsJavaMap(obj.asInstanceOf[StrMapValue].strMapValue)
      },
      classOf[IntToIntMapValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.mapAsJavaMap(obj.asInstanceOf[IntToIntMapValue].intToIntMapValue)
      },
      classOf[LongToIntMapValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.mapAsJavaMap(obj.asInstanceOf[LongToIntMapValue].longToIntMapValue)
      },
      classOf[ShortToDoubleMapValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.mapAsJavaMap(obj.asInstanceOf[ShortToDoubleMapValue].shortToDoubleMapValue)
      },
      classOf[LongToDoubleMapValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.mapAsJavaMap(obj.asInstanceOf[LongToDoubleMapValue].longToDoubleMapValue)
      },
      classOf[LongToStringMapValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.mapAsJavaMap(obj.asInstanceOf[LongToStringMapValue].longToStringMapValue)
      },
      classOf[IntToDoubleMapValue] -> { (obj: ScalaFeatureValue) =>
        WrapAsJava.mapAsJavaMap(obj.asInstanceOf[IntToDoubleMapValue].intToDoubleMapValue)
      },
      classOf[BinaryValue] -> { (obj: ScalaFeatureValue) =>
        obj.asInstanceOf[BinaryValue].binaryValue
      },
      classOf[StrListMapValue] -> { (obj: ScalaFeatureValue) =>
        toJava(obj.asInstanceOf[StrListMapValue].strListMapValue)
      },
      classOf[LongListMapValue] -> { (obj: ScalaFeatureValue) =>
        toJava(obj.asInstanceOf[LongListMapValue].longListMapValue)
      },
      classOf[IntListMapValue] -> { (obj: ScalaFeatureValue) =>
        toJava(obj.asInstanceOf[IntListMapValue].intListMapValue)
      },
      classOf[StrMapListValue] -> { (obj: ScalaFeatureValue) =>
        toJava(obj.asInstanceOf[StrMapListValue].strMapListValue)
      },
      classOf[LongToLongMapValue] -> { (obj: ScalaFeatureValue) =>
        toJava(obj.asInstanceOf[LongToLongMapValue].longToLongMapValue)
      },
      classOf[LongToStrSetMapValue] -> { (obj: ScalaFeatureValue) =>
        toJava(obj.asInstanceOf[LongToStrSetMapValue].longToStrSetMapValue)
      },
      classOf[LongToStrDoubleMapValue] -> { (obj: ScalaFeatureValue) =>
        toJava(obj.asInstanceOf[LongToStrDoubleMapValue].longToStrDoubleMapValue)
      },
      classOf[LongToBlobMapValue] -> { (obj: ScalaFeatureValue) =>
        toJava(obj.asInstanceOf[LongToBlobMapValue].longToBlobMapValue)
      }
    )

  val featureValueToJavaFeatureValueConvterMap: Map[AnyRef, ScalaFeatureValue => JavaFeatureValue] =
    Map[AnyRef, (ScalaFeatureValue) => JavaFeatureValue](
      classOf[ShortValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.shortValue(obj.asInstanceOf[ShortValue].shortValue)
      },
      classOf[IntValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.intValue(obj.asInstanceOf[IntValue].intValue)
      },
      classOf[LongValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.longValue(obj.asInstanceOf[LongValue].longValue)
      },
      classOf[DoubleValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.doubleValue(obj.asInstanceOf[DoubleValue].doubleValue)
      },
      classOf[BooleanValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.booleanValue(obj.asInstanceOf[BooleanValue].booleanValue)
      },
      classOf[StrValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.strValue(obj.asInstanceOf[StrValue].strValue)
      },
      classOf[ShortSetValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.shortSetValue(
          obj.asInstanceOf[ShortSetValue].shortSetValue.map(Short.box).asJava
        )
      },
      classOf[IntSetValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.intSetValue(obj.asInstanceOf[IntSetValue].intSetValue.map(Int.box).asJava)
      },
      classOf[LongSetValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.longSetValue(
          obj.asInstanceOf[LongSetValue].longSetValue.map(Long.box).asJava
        )
      },
      classOf[StrSetValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.strSetValue(obj.asInstanceOf[StrSetValue].strSetValue.asJava)
      },
      classOf[ShortListValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.shortListValue(
          obj.asInstanceOf[ShortListValue].shortListValue.map(Short.box).asJava
        )
      },
      classOf[IntListValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.intListValue(
          obj.asInstanceOf[IntListValue].intListValue.map(Int.box).asJava
        )
      },
      classOf[LongListValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.longListValue(
          obj.asInstanceOf[LongListValue].longListValue.map(Long.box).asJava
        )
      },
      classOf[DoubleListValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.doubleListValue(
          obj.asInstanceOf[DoubleListValue].doubleListValue.map(Double.box).asJava
        )
      },
      classOf[BooleanListValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.booleanListValue(
          obj.asInstanceOf[BooleanListValue].booleanListValue.map(Boolean.box).asJava
        )
      },
      classOf[StrListValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.strListValue(obj.asInstanceOf[StrListValue].strListValue.asJava)
      },
      classOf[ShortMapValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.shortMapValue(
          obj.asInstanceOf[ShortMapValue].shortMapValue.mapValues(Short.box).asJava
        )
      },
      classOf[IntMapValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.intMapValue(
          obj.asInstanceOf[IntMapValue].intMapValue.mapValues(Int.box).asJava
        )
      },
      classOf[LongMapValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.longMapValue(
          obj.asInstanceOf[LongMapValue].longMapValue.mapValues(Long.box).asJava
        )
      },
      classOf[DoubleMapValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.doubleMapValue(
          obj.asInstanceOf[DoubleMapValue].doubleMapValue.mapValues(Double.box).asJava
        )
      },
      classOf[BooleanMapValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.booleanMapValue(
          obj.asInstanceOf[BooleanMapValue].booleanMapValue.mapValues(Boolean.box).asJava
        )
      },
      classOf[StrMapValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.strMapValue(obj.asInstanceOf[StrMapValue].strMapValue.asJava)
      },
      classOf[IntToIntMapValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.intToIntMapValue(
          obj
            .asInstanceOf[IntToIntMapValue]
            .intToIntMapValue
            .map(t => Int.box(t._1) -> Int.box(t._2))
            .asJava
        )
      },
      classOf[LongToIntMapValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.longToIntMapValue(
          obj
            .asInstanceOf[LongToIntMapValue]
            .longToIntMapValue
            .map(t => Long.box(t._1) -> Int.box(t._2))
            .asJava
        )
      },
      classOf[ShortToDoubleMapValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.shortToDoubleMapValue(
          obj
            .asInstanceOf[ShortToDoubleMapValue]
            .shortToDoubleMapValue
            .map(t => Short.box(t._1) -> Double.box(t._2))
            .asJava
        )
      },
      classOf[LongToDoubleMapValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.longToDoubleMapValue(
          obj
            .asInstanceOf[LongToDoubleMapValue]
            .longToDoubleMapValue
            .map(t => Long.box(t._1) -> Double.box(t._2))
            .asJava
        )
      },
      classOf[LongToStringMapValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.longToStringMapValue(
          obj
            .asInstanceOf[LongToStringMapValue]
            .longToStringMapValue
            .map(t => Long.box(t._1) -> t._2)
            .asJava
        )
      },
      classOf[IntToDoubleMapValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.intToDoubleMapValue(
          obj
            .asInstanceOf[IntToDoubleMapValue]
            .intToDoubleMapValue
            .map(t => Int.box(t._1) -> Double.box(t._2))
            .asJava
        )
      },
      classOf[BinaryValue] -> { (obj: ScalaFeatureValue) =>
        JavaFeatureValue.binaryValue(obj.asInstanceOf[BinaryValue].binaryValue)
      },
      classOf[StrListMapValue] -> { (obj: ScalaFeatureValue) =>
        new JavaFeatureValue(
          JavaFeatureValue._Fields.STR_LIST_MAP_VALUE,
          toJava(obj.asInstanceOf[StrListMapValue].strListMapValue)
        )
      },
      classOf[LongListMapValue] -> { (obj: ScalaFeatureValue) =>
        new JavaFeatureValue(
          JavaFeatureValue._Fields.LONG_LIST_MAP_VALUE,
          toJava(obj.asInstanceOf[LongListMapValue].longListMapValue)
        )
      },
      classOf[IntListMapValue] -> { (obj: ScalaFeatureValue) =>
        new JavaFeatureValue(
          JavaFeatureValue._Fields.INT_LIST_MAP_VALUE,
          toJava(obj.asInstanceOf[IntListMapValue].intListMapValue)
        )
      },
      classOf[StrMapListValue] -> { (obj: ScalaFeatureValue) =>
        new JavaFeatureValue(
          JavaFeatureValue._Fields.STR_MAP_LIST_VALUE,
          toJava(obj.asInstanceOf[StrMapListValue].strMapListValue)
        )
      },
      classOf[LongToLongMapValue] -> { (obj: ScalaFeatureValue) =>
        new JavaFeatureValue(
          JavaFeatureValue._Fields.LONG_TO_LONG_MAP_VALUE,
          toJava(obj.asInstanceOf[LongToLongMapValue].longToLongMapValue)
        )
      },
      classOf[LongToStrSetMapValue] -> { (obj: ScalaFeatureValue) =>
        new JavaFeatureValue(
          JavaFeatureValue._Fields.LONG_TO_STR_SET_MAP_VALUE,
          toJava(obj.asInstanceOf[LongToStrSetMapValue].longToStrSetMapValue)
        )
      },
      classOf[LongToStrDoubleMapValue] -> { (obj: ScalaFeatureValue) =>
        new JavaFeatureValue(
          JavaFeatureValue._Fields.LONG_TO_STR_DOUBLE_MAP_VALUE,
          toJava(obj.asInstanceOf[LongToStrDoubleMapValue].longToStrDoubleMapValue)
        )
      },
      classOf[LongToBlobMapValue] -> { (obj: ScalaFeatureValue) =>
        new JavaFeatureValue(
          JavaFeatureValue._Fields.LONG_TO_BLOB_MAP_VALUE,
          toJava(obj.asInstanceOf[LongToBlobMapValue].longToBlobMapValue)
        )
      }
    )

  val featureTypeNameToBotMakerTypeMap: Map[String, Type] = Map[String, Type](
    "ShortValue" -> Type.SHORT,
    "IntValue" -> Type.INTEGER,
    "LongValue" -> Type.LONG,
    "DoubleValue" -> Type.DOUBLE,
    "BooleanValue" -> Type.BOOLEAN,
    "StrValue" -> Type.STRING,
    "ShortSetValue" -> Type.setOf(Type.SHORT),
    "IntSetValue" -> Type.setOf(Type.INTEGER),
    "LongSetValue" -> Type.setOf(Type.LONG),
    "StrSetValue" -> Type.setOf(Type.STRING),
    "ShortListValue" -> Type.listOf(Type.SHORT),
    "IntListValue" -> Type.listOf(Type.INTEGER),
    "LongListValue" -> Type.listOf(Type.LONG),
    "DoubleListValue" -> Type.listOf(Type.DOUBLE),
    "BooleanListValue" -> Type.listOf(Type.BOOLEAN),
    "StrListValue" -> Type.listOf(Type.STRING),
    "ShortMapValue" -> Type.mapOf(Type.STRING, Type.SHORT),
    "IntMapValue" -> Type.mapOf(Type.STRING, Type.INTEGER),
    "LongMapValue" -> Type.mapOf(Type.STRING, Type.LONG),
    "DoubleMapValue" -> Type.mapOf(Type.STRING, Type.DOUBLE),
    "BooleanMapValue" -> Type.mapOf(Type.STRING, Type.BOOLEAN),
    "StrMapValue" -> Type.mapOf(Type.STRING, Type.STRING),
    "IntToIntMapValue" -> Type.mapOf(Type.INTEGER, Type.INTEGER),
    "LongToIntMapValue" -> Type.mapOf(Type.LONG, Type.INTEGER),
    "ShortToDoubleMapValue" -> Type.mapOf(Type.SHORT, Type.DOUBLE),
    "LongToDoubleMapValue" -> Type.mapOf(Type.LONG, Type.DOUBLE),
    "LongToStringMapValue" -> Type.mapOf(Type.LONG, Type.STRING),
    "IntToDoubleMapValue" -> Type.mapOf(Type.INTEGER, Type.DOUBLE),
    "BinaryValue" -> Type.BINARY,
    "StrListMapValue" -> Type.mapOf(Type.STRING, Type.listOf(Type.STRING)),
    "LongListMapValue" -> Type.mapOf(Type.STRING, Type.listOf(Type.LONG)),
    "IntListMapValue" -> Type.mapOf(Type.STRING, Type.listOf(Type.INTEGER)),
    "StrMapListValue" -> Type.listOf(Type.mapOf(Type.STRING, Type.STRING)),
    "LongToStrSetMapValue" -> Type.mapOf(Type.LONG, Type.setOf(Type.STRING)),
    "LongToStrDoubleMapValue" -> Type.mapOf(Type.LONG, Type.mapOf(Type.STRING, Type.DOUBLE)),
    "LongToLongListMapValue" -> Type.mapOf(Type.LONG, Type.listOf(Type.LONG)),
    "LongToBlobMapValue" -> Type.mapOf(Type.LONG, Type.BINARY)
  )

  def typedObjectToFeatureData(value: AnyRef): JavaFeatureData = {
    val featureDataInjection = TypedFeatureDataInjection.of(
      ofType(value)
    )

    featureDataInjection.apply(value)
  }

  def featureDataToJavaObject(feature: Option[ScalaFeatureData]): AnyRef = {
    if (feature.isDefined && feature.get.featureValue.isDefined) {
      featureValueToJavaObject(feature.get.featureValue.get)
    } else {
      feature
    }
  }

  def featureValueToJavaObject(value: ScalaFeatureValue): AnyRef = {
    val converter = featureValueConvterMap.get(
      value.getClass
    )

    if (converter.isDefined) {
      converter.get.apply(value)
    } else {
      throw new RuntimeException(s"Unsupported FeatureData Type ${value.getClass}")
    }
  }

  def featureDataToJavaFeatureData(feature: ScalaFeatureData): JavaFeatureData = {
    val fd = new JavaFeatureData()
    if (feature.featureValue.isDefined) {
      fd.setFeatureValue(featureValueToJavaFeatureValue(feature.featureValue.get))
    }
    fd
  }

  def featureValueToJavaFeatureValue(value: ScalaFeatureValue): JavaFeatureValue = {
    val converter = featureValueToJavaFeatureValueConvterMap.get(
      value.getClass
    )

    if (converter.isDefined) {
      converter.get.apply(value)
    } else {
      throw new RuntimeException(s"Unsupported FeatureData Type ${value.getClass}")
    }
  }

  def toJava(m: AnyRef): AnyRef = {
    m match {
      case sm: Map[_, AnyRef] => WrapAsJava.mapAsJavaMap(sm.map(kv => (kv._1, toJava(kv._2))))
      case st: Set[AnyRef] => WrapAsJava.setAsJavaSet(st.map(k => toJava(k)))
      case sq: Seq[AnyRef] => WrapAsJava.seqAsJavaList(sq.map(k => toJava(k)))
      case _ => m
    }
  }

  def getBotMakerType(featureTypeName: String): Type = {
    val typeOpt: Option[Type] = featureTypeNameToBotMakerTypeMap.get(
      featureTypeName
    )

    if (typeOpt.isDefined) {
      typeOpt.get
    } else {
      throw new RuntimeException("Unsupported FeatureData Type")
    }
  }

  val javaFeatureValueType: ThriftType = Type.thriftOf(classOf[JavaFeatureValue])
  val javaFeatureDataType: ThriftType = Type.thriftOf(classOf[JavaFeatureData])
  val javaFeatureValueConverters: Map[Type, AnyRef => JavaFeatureData] = {
    val converters = javaFeatureValueType.getFieldNames.asScala map { fieldName =>
      val fieldType = javaFeatureValueType.getFieldMetadata(fieldName).valueMetaData.getRawType
      (
        fieldType,
        (value: AnyRef) => {
          val featureValue = new JavaFeatureValue()
          javaFeatureValueType.setFieldValue(featureValue, fieldName, value)
          new JavaFeatureData().setFeatureValue(featureValue)
        })
    }

    (
      (javaFeatureDataType, (value: AnyRef) => value.asInstanceOf[JavaFeatureData]) :: (
        javaFeatureValueType,
        (value: AnyRef) =>
          new JavaFeatureData().setFeatureValue(value.asInstanceOf[JavaFeatureValue])
      ) :: converters.toList
    ).toMap
  }

  val javaFeatureValueFieldIdToTypeMapping: Map[Short, Type] = {
    javaFeatureValueType.getFieldNames.asScala.map { fieldName =>
      val fieldMetadata = javaFeatureValueType.getFieldMetadata(fieldName)
      fieldMetadata.fieldId -> fieldMetadata.valueMetaData.getType
    }.toMap
  }

  val scalaFeatureValueType: ThriftStructType = Type.thriftStructOf(classOf[ScalaFeatureValue])
  val scalaFeatureDataType: ThriftStructType = Type.thriftStructOf(classOf[ScalaFeatureData])
  val scalaFeatureValueConverters: Map[Type, AnyRef => ScalaFeatureData] = {
    val converters = scalaFeatureValueType.getFieldNames.asScala map { fieldName =>
      val fieldType = scalaFeatureValueType.getFieldMetadata(fieldName).valueMetaData.getRawType
      (
        fieldType,
        (value: AnyRef) => {
          val featureValue = scalaFeatureValueType
            .newInstance(ImmutableMap.of(fieldName, value))
            .asInstanceOf[ScalaFeatureValue]
          ScalaFeatureData(featureValue = Some(featureValue))
        })
    }

    (
      (scalaFeatureDataType, (value: AnyRef) => value.asInstanceOf[ScalaFeatureData]) :: (
        scalaFeatureValueType,
        (value: AnyRef) =>
          ScalaFeatureData(featureValue = Some(value.asInstanceOf[ScalaFeatureValue]))
      ) :: converters.toList
    ).toMap
  }

  def toJava(ft: Type, fv: AnyRef): JavaFeatureData = {
    javaFeatureValueConverters
      .get(ft)
      .map(converter => converter(fv))
      .getOrElse(BotMakerFeatures.makeFeatureData(fv))
  }

  def toJavaWithDebugInfo(ft: Type, fv: AnyRef): JavaFeatureData = {
    try {
      toJava(ft, fv)
    } catch {
      case t: Throwable =>
        new JavaFeatureData().setDebugInfo(t.getMessage)
    }
  }

  def toScala(ft: Type, fv: AnyRef): ScalaFeatureData = {
    scalaFeatureValueConverters
      .get(ft)
      .map(converter => converter(fv))
      .getOrElse(throw new RuntimeFailure(s"Unsupported FeatureData Type $ft"))
  }

  def toScalaWithDebugInfo(ft: Type, fv: AnyRef): ScalaFeatureData = {
    try {
      toScala(ft, fv)
    } catch {
      case t: Throwable =>
        ScalaFeatureData(debugInfo = Some(t.getMessage))
    }
  }
}
