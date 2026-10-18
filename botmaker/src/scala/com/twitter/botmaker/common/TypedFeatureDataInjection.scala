package com.twitter.botmaker.common

import com.google.common.collect.ImmutableList
import com.google.common.collect.ImmutableMap
import com.google.common.collect.ImmutableSet
import com.twitter.bijection.AbstractInjection
import com.twitter.bijection.Injection
import com.twitter.botmaker.compiler.types.Type
import com.twitter.relevance.feature_store.thrift.FeatureData
import com.twitter.relevance.feature_store.thrift.FeatureValue
import java.util.{List => JList}
import java.util.{Map => JMap}
import java.util.{Set => JSet}
import scala.collection.JavaConverters._
import scala.util.Failure
import scala.util.Try

abstract class OneWayInjection[A, B] extends AbstractInjection[A, B] {
  override def invert(b: B): Try[A] = {
    Failure[A](new IllegalArgumentException(getClass + " does not allow invert()"))
  }
}

class ObjectToBasicTypeInjection[T](val clazz: Class[T]) extends OneWayInjection[AnyRef, T] {
  def apply(obj: AnyRef): T = {
    if (clazz eq classOf[String]) {
      obj.toString.asInstanceOf[T]
    } else if (clazz eq classOf[Long]) {
      convertToInteger(obj)
    } else if (clazz eq classOf[Double]) {
      if (obj.isInstanceOf[Double]) {
        obj.asInstanceOf[T]
      } else {
        obj.toString.toDouble.asInstanceOf[T]
      }
    } else if (clazz eq classOf[Int]) {
      convertToInteger(obj)
    } else if (clazz eq classOf[Short]) {
      convertToInteger(obj)
    } else if (clazz eq classOf[Byte]) {
      convertToInteger(obj)
    } else {
      obj.asInstanceOf[T]
    }
  }

  private def convertToInteger(obj: AnyRef): T = {
    try {
      obj match {
        case t: T =>
          t
        case _ =>
          obj.toString.toLong.asInstanceOf[T]
      }
    } catch {
      case _: NumberFormatException =>
        obj.toString.toDouble.toLong.asInstanceOf[T]
    }
  }
}

class IterableToListInjection[T](val itemInjection: Injection[AnyRef, T])
    extends OneWayInjection[java.lang.Iterable[AnyRef], JList[T]] {

  def apply(iterable: java.lang.Iterable[AnyRef]): JList[T] = {
    val builder: ImmutableList.Builder[T] =
      ImmutableList.builder.asInstanceOf[ImmutableList.Builder[T]]

    for (item <- iterable.asScala) {
      builder.add(
        itemInjection.apply(item)
      )
    }

    builder.build
  }
}

class IterableToSetInjection[T](val itemInjection: Injection[AnyRef, T])
    extends OneWayInjection[java.lang.Iterable[AnyRef], JSet[T]] {

  def apply(iterable: java.lang.Iterable[AnyRef]): JSet[T] = {
    val builder: ImmutableSet.Builder[T] =
      ImmutableSet.builder.asInstanceOf[ImmutableSet.Builder[T]]

    for (item <- iterable.asScala) {
      builder.add(
        itemInjection.apply(item)
      )
    }

    builder.build
  }
}

class MapEntryConversionInjection[K, V](
  val keyInjection: Injection[AnyRef, K],
  val valueInjection: Injection[AnyRef, V])
    extends OneWayInjection[JMap[AnyRef, AnyRef], JMap[K, V]] {

  def apply(map: JMap[AnyRef, AnyRef]): JMap[K, V] = {
    val builder: ImmutableMap.Builder[K, V] =
      ImmutableMap.builder.asInstanceOf[ImmutableMap.Builder[K, V]]

    for (entry <- map.entrySet.asScala) {
      builder.put(
        keyInjection.apply(entry.getKey),
        valueInjection.apply(entry.getValue)
      )
    }

    builder.build
  }
}

object TypedFeatureDataInjection {

  val shortInjection: ObjectToBasicTypeInjection[Short] =
    new ObjectToBasicTypeInjection[Short](classOf[Short])
  val intInjection: ObjectToBasicTypeInjection[Int] =
    new ObjectToBasicTypeInjection[Int](classOf[Int])
  val longInjection: ObjectToBasicTypeInjection[Long] =
    new ObjectToBasicTypeInjection[Long](classOf[Long])
  val doubleInjection: ObjectToBasicTypeInjection[Double] =
    new ObjectToBasicTypeInjection[Double](classOf[Double])
  val boolInjection: ObjectToBasicTypeInjection[Boolean] =
    new ObjectToBasicTypeInjection[Boolean](classOf[Boolean])
  val strInjection: ObjectToBasicTypeInjection[String] =
    new ObjectToBasicTypeInjection[String](classOf[String])

  val shortListInjection: IterableToListInjection[Short] = new IterableToListInjection(
    shortInjection)
  val intListInjection: IterableToListInjection[Int] = new IterableToListInjection(intInjection)
  val longListInjection: IterableToListInjection[Long] = new IterableToListInjection(longInjection)
  val doubleListInjection: IterableToListInjection[Double] = new IterableToListInjection(
    doubleInjection)
  val boolListInjection: IterableToListInjection[Boolean] = new IterableToListInjection(
    boolInjection)
  val strListInjection: IterableToListInjection[String] = new IterableToListInjection(strInjection)

  val shortSetInjection: IterableToSetInjection[Short] = new IterableToSetInjection(shortInjection)
  val intSetInjection: IterableToSetInjection[Int] = new IterableToSetInjection(intInjection)
  val longSetInjection: IterableToSetInjection[Long] = new IterableToSetInjection(longInjection)
  val strSetInjection: IterableToSetInjection[String] = new IterableToSetInjection(strInjection)

  val longToDoubleMapInjection: MapEntryConversionInjection[Long, Double] =
    new MapEntryConversionInjection(
      longInjection,
      doubleInjection
    )
  val longToLongMapInjection: MapEntryConversionInjection[Long, Long] =
    new MapEntryConversionInjection(
      longInjection,
      longInjection
    )
  val longToIntMapInjection: MapEntryConversionInjection[Long, Int] =
    new MapEntryConversionInjection(
      longInjection,
      intInjection
    )
  val longToStrMapInjection: MapEntryConversionInjection[Long, String] =
    new MapEntryConversionInjection(
      longInjection,
      strInjection
    )
  val intToDoubleMapInjection: MapEntryConversionInjection[Int, Double] =
    new MapEntryConversionInjection(
      intInjection,
      doubleInjection
    )
  val shortToDoubleMapInjection: MapEntryConversionInjection[Short, Double] =
    new MapEntryConversionInjection(
      shortInjection,
      doubleInjection
    )
  val intToIntMapInjection: MapEntryConversionInjection[Int, Int] = new MapEntryConversionInjection(
    intInjection,
    intInjection
  )

  val strToLongListMapInjection: MapEntryConversionInjection[String, JList[Long]] =
    new MapEntryConversionInjection(
      strInjection,
      longListInjection.asInstanceOf[Injection[AnyRef, JList[Long]]]
    )
  val strToStrListMapInjection: MapEntryConversionInjection[String, JList[String]] =
    new MapEntryConversionInjection(
      strInjection,
      strListInjection.asInstanceOf[Injection[AnyRef, JList[String]]]
    )
  val strToShortMapInjection: MapEntryConversionInjection[String, Short] =
    new MapEntryConversionInjection(
      strInjection,
      shortInjection
    )
  val strToIntMapInjection: MapEntryConversionInjection[String, Int] =
    new MapEntryConversionInjection(
      strInjection,
      intInjection
    )
  val strToLongMapInjection: MapEntryConversionInjection[String, Long] =
    new MapEntryConversionInjection(
      strInjection,
      longInjection
    )
  val strToDoubleMapInjection: MapEntryConversionInjection[String, Double] =
    new MapEntryConversionInjection(
      strInjection,
      doubleInjection
    )
  val strToBoolMapInjection: MapEntryConversionInjection[String, Boolean] =
    new MapEntryConversionInjection(
      strInjection,
      boolInjection
    )
  val strToStrMapInjection: MapEntryConversionInjection[String, String] =
    new MapEntryConversionInjection(
      strInjection,
      strInjection
    )

  val DefaultFeatureInjection: OneWayInjection[AnyRef, FeatureData] = makeFeatureDataInjection(
    strInjection,
    FeatureValue._Fields.STR_VALUE
  )

  val BuildinInjection: Map[Type, OneWayInjection[_, FeatureData]] = Map(
    Type.listOf(Type.SHORT) -> makeFeatureDataInjection(
      shortListInjection,
      FeatureValue._Fields.SHORT_LIST_VALUE
    ),
    Type.listOf(Type.INTEGER) -> makeFeatureDataInjection(
      intListInjection,
      FeatureValue._Fields.INT_LIST_VALUE
    ),
    Type.listOf(Type.LONG) -> makeFeatureDataInjection(
      longListInjection,
      FeatureValue._Fields.LONG_LIST_VALUE
    ),
    Type.listOf(Type.NUMBER) -> makeFeatureDataInjection(
      doubleListInjection,
      FeatureValue._Fields.DOUBLE_LIST_VALUE
    ),
    Type.listOf(Type.DOUBLE) -> makeFeatureDataInjection(
      doubleListInjection,
      FeatureValue._Fields.DOUBLE_LIST_VALUE
    ),
    Type.listOf(Type.BOOLEAN) -> makeFeatureDataInjection(
      boolListInjection,
      FeatureValue._Fields.BOOLEAN_LIST_VALUE
    ),
    Type.listOf(Type.STRING) -> makeFeatureDataInjection(
      strListInjection,
      FeatureValue._Fields.STR_LIST_VALUE
    ),
    Type.listOf(Type.OBJECT) -> makeFeatureDataInjection(
      strListInjection,
      FeatureValue._Fields.STR_LIST_VALUE
    ),
    Type.setOf(Type.SHORT) -> makeFeatureDataInjection(
      shortSetInjection,
      FeatureValue._Fields.SHORT_SET_VALUE
    ),
    Type.setOf(Type.INTEGER) -> makeFeatureDataInjection(
      intSetInjection,
      FeatureValue._Fields.INT_SET_VALUE
    ),
    Type.setOf(Type.LONG) -> makeFeatureDataInjection(
      longSetInjection,
      FeatureValue._Fields.LONG_SET_VALUE
    ),
    Type.setOf(Type.STRING) -> makeFeatureDataInjection(
      strSetInjection,
      FeatureValue._Fields.STR_SET_VALUE
    ),
    Type.setOf(Type.OBJECT) -> makeFeatureDataInjection(
      strSetInjection,
      FeatureValue._Fields.STR_SET_VALUE
    ),
    Type.mapOf(Type.SHORT, Type.DOUBLE) -> makeFeatureDataInjection(
      shortToDoubleMapInjection,
      FeatureValue._Fields.SHORT_TO_DOUBLE_MAP_VALUE
    ),
    Type.mapOf(Type.INTEGER, Type.INTEGER) -> makeFeatureDataInjection(
      intToIntMapInjection,
      FeatureValue._Fields.INT_TO_INT_MAP_VALUE
    ),
    Type.mapOf(Type.INTEGER, Type.DOUBLE) -> makeFeatureDataInjection(
      intToDoubleMapInjection,
      FeatureValue._Fields.INT_TO_DOUBLE_MAP_VALUE
    ),
    Type.mapOf(Type.LONG, Type.INTEGER) -> makeFeatureDataInjection(
      longToIntMapInjection,
      FeatureValue._Fields.LONG_TO_INT_MAP_VALUE
    ),
    Type.mapOf(Type.LONG, Type.LONG) -> makeFeatureDataInjection(
      longToLongMapInjection,
      FeatureValue._Fields.LONG_TO_LONG_MAP_VALUE
    ),
    Type.mapOf(Type.LONG, Type.NUMBER) -> makeFeatureDataInjection(
      longToDoubleMapInjection,
      FeatureValue._Fields.LONG_TO_DOUBLE_MAP_VALUE
    ),
    Type.mapOf(Type.LONG, Type.DOUBLE) -> makeFeatureDataInjection(
      longToDoubleMapInjection,
      FeatureValue._Fields.LONG_TO_DOUBLE_MAP_VALUE
    ),
    Type.mapOf(Type.LONG, Type.OBJECT) -> makeFeatureDataInjection(
      longToStrMapInjection,
      FeatureValue._Fields.LONG_TO_STRING_MAP_VALUE
    ),
    Type.mapOf(Type.STRING, Type.listOf(Type.LONG)) -> makeFeatureDataInjection(
      strToLongListMapInjection,
      FeatureValue._Fields.LONG_LIST_MAP_VALUE
    ),
    Type.mapOf(Type.STRING, Type.listOf(Type.OBJECT)) -> makeFeatureDataInjection(
      strToStrListMapInjection,
      FeatureValue._Fields.STR_LIST_MAP_VALUE
    ),
    Type.mapOf(Type.STRING, Type.SHORT) -> makeFeatureDataInjection(
      strToShortMapInjection,
      FeatureValue._Fields.SHORT_MAP_VALUE
    ),
    Type.mapOf(Type.STRING, Type.INTEGER) -> makeFeatureDataInjection(
      strToIntMapInjection,
      FeatureValue._Fields.INT_MAP_VALUE
    ),
    Type.mapOf(Type.STRING, Type.LONG) -> makeFeatureDataInjection(
      strToLongMapInjection,
      FeatureValue._Fields.LONG_MAP_VALUE
    ),
    Type.mapOf(Type.STRING, Type.NUMBER) -> makeFeatureDataInjection(
      strToDoubleMapInjection,
      FeatureValue._Fields.DOUBLE_MAP_VALUE
    ),
    Type.mapOf(Type.STRING, Type.DOUBLE) -> makeFeatureDataInjection(
      strToDoubleMapInjection,
      FeatureValue._Fields.DOUBLE_MAP_VALUE
    ),
    Type.mapOf(Type.STRING, Type.BOOLEAN) -> makeFeatureDataInjection(
      strToBoolMapInjection,
      FeatureValue._Fields.BOOLEAN_MAP_VALUE
    ),
    Type.mapOf(Type.STRING, Type.STRING) -> makeFeatureDataInjection(
      strToStrMapInjection,
      FeatureValue._Fields.STR_MAP_VALUE
    ),
    Type.mapOf(Type.STRING, Type.NUMBER) -> makeFeatureDataInjection(
      strToDoubleMapInjection,
      FeatureValue._Fields.DOUBLE_MAP_VALUE
    ),
    Type.mapOf(Type.STRING, Type.DOUBLE) -> makeFeatureDataInjection(
      strToDoubleMapInjection,
      FeatureValue._Fields.DOUBLE_MAP_VALUE
    ),
    Type.mapOf(Type.OBJECT, Type.listOf(Type.LONG)) -> makeFeatureDataInjection(
      strToLongListMapInjection,
      FeatureValue._Fields.LONG_LIST_MAP_VALUE
    ),
    Type.mapOf(Type.OBJECT, Type.listOf(Type.OBJECT)) -> makeFeatureDataInjection(
      strToStrListMapInjection,
      FeatureValue._Fields.STR_LIST_MAP_VALUE
    ),
    Type.mapOf(Type.OBJECT, Type.INTEGER) -> makeFeatureDataInjection(
      strToIntMapInjection,
      FeatureValue._Fields.INT_MAP_VALUE
    ),
    Type.mapOf(Type.OBJECT, Type.LONG) -> makeFeatureDataInjection(
      strToLongMapInjection,
      FeatureValue._Fields.LONG_MAP_VALUE
    ),
    Type.mapOf(Type.OBJECT, Type.LONG) -> makeFeatureDataInjection(
      strToLongMapInjection,
      FeatureValue._Fields.LONG_MAP_VALUE
    ),
    Type.mapOf(Type.OBJECT, Type.NUMBER) -> makeFeatureDataInjection(
      strToDoubleMapInjection,
      FeatureValue._Fields.DOUBLE_MAP_VALUE
    ),
    Type.mapOf(Type.OBJECT, Type.DOUBLE) -> makeFeatureDataInjection(
      strToDoubleMapInjection,
      FeatureValue._Fields.DOUBLE_MAP_VALUE
    ),
    Type.mapOf(Type.OBJECT, Type.BOOLEAN) -> makeFeatureDataInjection(
      strToBoolMapInjection,
      FeatureValue._Fields.BOOLEAN_MAP_VALUE
    ),
    Type.mapOf(Type.OBJECT, Type.OBJECT) -> makeFeatureDataInjection(
      strToStrMapInjection,
      FeatureValue._Fields.STR_MAP_VALUE
    ),
    Type.SHORT -> makeFeatureDataInjection(
      shortInjection,
      FeatureValue._Fields.SHORT_VALUE
    ),
    Type.INTEGER -> makeFeatureDataInjection(
      intInjection,
      FeatureValue._Fields.INT_VALUE
    ),
    Type.LONG -> makeFeatureDataInjection(
      longInjection,
      FeatureValue._Fields.LONG_VALUE
    ),
    Type.DOUBLE -> makeFeatureDataInjection(
      doubleInjection,
      FeatureValue._Fields.DOUBLE_VALUE
    ),
    Type.NUMBER -> makeFeatureDataInjection(
      doubleInjection,
      FeatureValue._Fields.DOUBLE_VALUE
    ),
    Type.BOOLEAN -> makeFeatureDataInjection(
      boolInjection,
      FeatureValue._Fields.BOOLEAN_VALUE
    ),
    Type.STRING -> makeFeatureDataInjection(
      strInjection,
      FeatureValue._Fields.STR_VALUE
    )
  )

  def of[T](botmakerType: Type): Injection[AnyRef, FeatureData] = {
    if (BuildinInjection.contains(botmakerType)) {
      BuildinInjection(botmakerType)
        .asInstanceOf[Injection[AnyRef, FeatureData]]
    } else {

      val injections = BuildinInjection.filter { x => Type.isSuperOf(x._1, botmakerType) }

      if (injections.nonEmpty) {
        injections.headOption.get._2
          .asInstanceOf[Injection[AnyRef, FeatureData]]
      } else {
        DefaultFeatureInjection
      }
    }
  }

  private def makeFeatureDataInjection[AnyRef, T](
    injection: Injection[AnyRef, T],
    fieldsEnum: FeatureValue._Fields
  ) = {
    new OneWayInjection[AnyRef, FeatureData]() {

      def apply(obj: AnyRef): FeatureData = {
        new FeatureData().setFeatureValue(
          new FeatureValue(fieldsEnum, injection.apply(obj))
        )
      }
    }
  }
}
