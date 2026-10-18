package com.twitter.botmaker.util

import com.twitter.botmaker.common.SerializerUtil
import com.twitter.botmaker.rule.thriftscala.BotMakerDerivedFeature
import com.twitter.botmaker.rule.thriftscala.BotMakerRulesPackage
import com.twitter.scrooge.ThriftStruct
import com.twitter.scrooge.ThriftStructCodec
import com.twitter.util.Memoize
import java.io._
import java.lang.invoke.MethodHandles
import java.lang.invoke.MethodType
import java.lang.reflect.Constructor
import java.util.{List => JList}
import java.util.{Map => JMap}
import java.util.{Set => JSet}
import org.apache.thrift.meta_data.FieldMetaData
import org.apache.thrift.meta_data.FieldValueMetaData
import org.apache.thrift.meta_data.ListMetaData
import org.apache.thrift.meta_data.MapMetaData
import org.apache.thrift.meta_data.SetMetaData
import org.apache.thrift.protocol.TBinaryProtocol
import org.apache.thrift.protocol.TType
import org.apache.thrift.transport.TIOStreamTransport
import org.apache.thrift.transport.TTransportException
import org.apache.thrift.TBase
import org.apache.thrift.TFieldIdEnum
import org.apache.thrift.TUnion
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import scala.collection.JavaConverters._
import scala.collection.mutable
import scala.collection.mutable.ListBuffer
import scala.language.existentials
import scala.reflect.ClassTag

object ThriftUtil {

  private[this] val Logger: Logger = LoggerFactory.getLogger(getClass)

  type Thrift = TBase[T, F] forSome {
    type F <: TFieldIdEnum
    type T <: TBase[T, F]
  }

  final val ThriftAnnotationsField = "fieldAnnotations"
  final val PublicFieldAnnotationKey = "keep_when_filter"
  final val PublicFieldAnnotationValue = "true"

  def mergeBotMakerRulesPackageFromJson(
    oldJson: String,
    newJson: String,
    defaultRuleOffset: Int = 100000,
    exceptionOnNameConflict: Boolean = true
  ): BotMakerRulesPackage = {
    val oldPkg = SerializerUtil.fromJson(BotMakerRulesPackage, oldJson)
    val newPkg = SerializerUtil.fromJson(BotMakerRulesPackage, newJson)

    val oldDfMap: Map[String, BotMakerDerivedFeature] =
      oldPkg.derivedFeatures.map(df => (df.name, df)).toMap

    val rules = oldPkg.rules ++ newPkg.rules.map(r => r.copy(id = r.id + defaultRuleOffset))
    val dfMap = mutable.Map[String, BotMakerDerivedFeature]()
    dfMap ++= oldDfMap
    newPkg.derivedFeatures.foreach { df =>
      if (oldDfMap.contains(df.name) && exceptionOnNameConflict)
        throw new IllegalArgumentException(s"Package 2 has conflicting DF name:: ${df.name}")
      else dfMap(df.name) = df
    }
    val dfs = dfMap.values.toList

    BotMakerRulesPackage(
      rules = rules,
      derivedFeatures = dfs
    )
  }

  def read[T <: ThriftStruct](file: String, codec: ThriftStructCodec[T]): List[T] = {

    val inputStream = new FileInputStream(file)
    val transport = new TIOStreamTransport(inputStream)
    val protocol = new TBinaryProtocol(transport)
    var list: ListBuffer[T] = ListBuffer()

    try {
      while (true) {
        list += codec.decode(protocol)
      }
    } catch {
      case _: TTransportException => ()
    } finally {
      transport.close()
    }

    list.toList
  }

  def readText[T <: ThriftStruct](file: String, codec: ThriftStructCodec[T]): List[T] = {
    scala.io.Source
      .fromFile(file)
      .getLines()
      .map(line => SerializerUtil.fromBytes(codec, SerializerUtil.fromBase64String(line)))
      .toList
  }

  def readJson[T <: ThriftStruct](file: String, codec: ThriftStructCodec[T]): List[T] = {
    scala.io.Source
      .fromFile(file)
      .getLines()
      .map(line => SerializerUtil.fromJson(codec, line))
      .toList
  }

  def write[T <: ThriftStruct](file: String, data: T): Unit = {
    write(file, List(data))
  }

  def write[T <: ThriftStruct](file: String, data: Seq[T]): Unit = {

    val outputStream = new FileOutputStream(file, true)
    val transport = new TIOStreamTransport(outputStream)
    val protocol = new TBinaryProtocol(transport)

    try {
      data.foreach(_.write(protocol))
    } finally {
      transport.close()
    }
  }

  def writeText[T <: ThriftStruct](
    file: String,
    data: T
  )(
    implicit codec: ThriftStructCodec[T]
  ): Unit = {
    writeText(file, List[T](data))(codec)
  }

  def writeText[T <: ThriftStruct](
    file: String,
    data: Seq[T]
  )(
    implicit codec: ThriftStructCodec[T]
  ): Unit = {
    FileUtil.writeAllLines(
      file,
      data.map(x => SerializerUtil.toBase64String(SerializerUtil.toBytes(codec, x))))
  }

  def writeJson[T <: ThriftStruct](
    file: String,
    data: T
  )(
    implicit codec: ThriftStructCodec[T]
  ): Unit = {
    writeJson(file, List(data))(codec)
  }

  def writeJson[T <: ThriftStruct](
    file: String,
    data: Seq[T]
  )(
    implicit codec: ThriftStructCodec[T]
  ): Unit = {
    FileUtil.writeAllLines(file, data.map(x => SerializerUtil.toJson(codec, x)))
  }

  def read[T <: TBase[_, _]](file: String)(implicit classTag: ClassTag[T]): List[T] = {
    val inputStream = new FileInputStream(file)
    val transport = new TIOStreamTransport(inputStream)
    val protocol = new TBinaryProtocol(transport)
    var list: ListBuffer[T] = ListBuffer()

    try {
      while (true) {
        val obj = classTag.runtimeClass.newInstance.asInstanceOf[T]
        obj.read(protocol)
        list += obj
      }
    } catch {
      case _: TTransportException => ()
    } finally {
      transport.close()
    }

    list.toList
  }

  def write[T <: TBase[_, _]](file: String, data: T*)(implicit classTag: ClassTag[T]): Unit = {

    val outputStream = new FileOutputStream(file, true)
    val transport = new TIOStreamTransport(outputStream)
    val protocol = new TBinaryProtocol(transport)

    try {
      data.foreach(_.write(protocol))
    } finally {
      transport.close()
    }
  }

  def getBytes(buf: java.nio.ByteBuffer): Array[Byte] = {
    val bytes = new Array[Byte](buf.remaining)
    buf.get(bytes)
    bytes
  }

  def getOrElse[F <: TFieldIdEnum, T <: TBase[T, F], D](thrift: T, field: F, default: D): D = {
    if (thrift.isSet(field)) {
      thrift.getFieldValue(field).asInstanceOf[D]
    } else {
      default
    }
  }

  def filterThrift[T <: TBase[T, F], F <: TFieldIdEnum](thrift: TBase[T, F]): TBase[T, F] = {
    val output: T = newInstance(thrift.getClass).asInstanceOf[T]
    val metadataMap: Map[F, FieldMetaData] =
      FieldMetaData
        .getStructMetaDataMap(thrift.getClass).asScala.toMap
        .asInstanceOf[Map[F, FieldMetaData]]
    for (fieldId <- metadataMap.keys) {
      if (thrift.isSet(fieldId) && isPublic(thrift.getClass, fieldId)) {
        output.setFieldValue(
          fieldId,
          filterValue(thrift.getFieldValue(fieldId), metadataMap(fieldId).valueMetaData)
        )
      }
    }
    output
  }

  def removeSetField[T <: TBase[T, F], F <: TFieldIdEnum](
    thrift: TBase[T, F],
    fields: Seq[String]
  ): TBase[T, F] = {

    fields.headOption match {
      case Some(fieldToCheck) =>
        thriftIdAndMetadataByFieldNameMemo(thrift.getClass).get(fieldToCheck) match {
          case Some((fieldId: F, fieldMetaData)) =>
            if (!thrift.isSet(fieldId)) {
              thrift
            } else if (fields.lengthCompare(1) == 0) {
              if (thrift.isInstanceOf[TUnion[_, _]]) {
                null
              } else {
                val newThrift = copy(thrift)
                newThrift.setFieldValue(fieldId, null)
                newThrift
              }
            } else {
              val nextFields = fields.drop(1)
              val nextValue = thrift.getFieldValue(fieldId)
              val valueMetaData = fieldMetaData.valueMetaData
              val transformedValue = (nextValue, valueMetaData.`type`) match {
                case (null, _) => null
                case (_, TType.STRUCT) =>
                  removeSetField(nextValue.asInstanceOf[TBase[T, F]], nextFields)
                case (_, TType.SET) =>
                  val setMetadata = valueMetaData.asInstanceOf[SetMetaData]
                  if (setMetadata.elemMetaData.isStruct) {
                    nextValue
                      .asInstanceOf[JSet[_]].asScala.map(elem =>
                        removeSetField(elem.asInstanceOf[TBase[T, F]], nextFields)).asJava
                  } else {
                    nextValue
                  }
                case (_, TType.LIST) =>
                  val listMetaData = valueMetaData.asInstanceOf[ListMetaData]
                  if (listMetaData.elemMetaData.isStruct) {
                    nextValue
                      .asInstanceOf[JList[_]].asScala.map(elem =>
                        removeSetField(elem.asInstanceOf[TBase[T, F]], nextFields)).asJava
                  } else {
                    nextValue
                  }
                case (_, TType.MAP) =>
                  nextValue
                case _ => nextValue
              }
              if (nextValue ne transformedValue) {
                val newThrift = copy(thrift)
                newThrift.setFieldValue(fieldId, transformedValue)
                newThrift
              } else {
                thrift
              }
            }

          case _ =>
            thrift
        }
      case _ => thrift
    }
  }

  private def copy[T <: TBase[T, F], F <: TFieldIdEnum](
    source: TBase[T, F]
  ): T = thriftDeepCopyMemo(source.getClass)(source).asInstanceOf[T]

  def extractThriftFields[T <: Thrift](
    thriftClass: Class[T]
  ): Set[String] = {
    val metadataMap = FieldMetaData.getStructMetaDataMap(thriftClass).asScala
    metadataMap.values.map(_.fieldName).toSet
  }

  private val thriftConstructorMemo: Class[_] => Constructor[_] =
    Memoize { clazz: Class[_] =>
      val constructor = clazz.getDeclaredConstructor(Seq.empty[Class[_]]: _*)
      constructor.setAccessible(true)
      constructor
    }

  private val thriftDeepCopyMemo: Class[_ <: Thrift] => AnyRef => AnyRef = Memoize {
    clazz: Class[_ <: Thrift] =>
      try {
        val methodType = MethodType.methodType(clazz)
        val methodHandle = MethodHandles.publicLookup.findVirtual(clazz, "deepCopy", methodType)
        inputThrift => methodHandle.invoke(inputThrift)
      } catch {
        case e: Exception =>
          Logger.warn(s"Unable to resolve deepCopy for ${clazz.getName}, defaulting to no-op", e)
          inputThrift => inputThrift
      }
  }

  private val thriftFieldAnnotationsMemo: Class[_] => Map[TFieldIdEnum, Map[String, String]] =
    Memoize { clazz: Class[_] =>
      try {
        clazz
          .getField(ThriftAnnotationsField).get(null)
          .asInstanceOf[JMap[TFieldIdEnum, JMap[String, String]]]
          .asScala.mapValues(_.asScala.toMap).toMap
      } catch {
        case _: Throwable => Map.empty
      }
    }

  private val thriftIdAndMetadataByFieldNameMemo: Class[_ <: Thrift] => Map[
    String,
    (TFieldIdEnum, FieldMetaData)
  ] = Memoize { clazz: Class[_ <: Thrift] =>
    FieldMetaData.getStructMetaDataMap(clazz).asScala.toMap.map {
      case (k, v) => k.getFieldName -> (k, v)
    }
  }

  private def filterValue(value: Any, metaData: FieldValueMetaData): Any = {
    (value, metaData.`type`) match {
      case (null, _) => value
      case (_, TType.STRUCT) => filterThrift(value.asInstanceOf[Thrift])
      case (_, TType.MAP) =>
        filterMap(value.asInstanceOf[JMap[_, _]], metaData.asInstanceOf[MapMetaData])
      case (_, TType.SET) =>
        filterSet(value.asInstanceOf[JSet[_]], metaData.asInstanceOf[SetMetaData])
      case (_, TType.LIST) =>
        filterList(value.asInstanceOf[JList[_]], metaData.asInstanceOf[ListMetaData])
      case _ => value
    }
  }

  private def filterMap(value: JMap[_, _], metaData: MapMetaData): JMap[_, _] = {
    value.asScala
      .map {
        case (k, v) =>
          (filterValue(k, metaData.keyMetaData), filterValue(v, metaData.valueMetaData))
      }.toMap.asJava
  }

  private def filterSet(value: JSet[_], metaData: SetMetaData): JSet[_] = {
    value.asScala.map { v => filterValue(v, metaData.elemMetaData) }.asJava
  }

  private def filterList(value: JList[_], metaData: ListMetaData): JList[_] = {
    value.asScala.map { v => filterValue(v, metaData.elemMetaData) }.asJava
  }

  private def newInstance(clazz: Class[_]): Any = {
    val constructor: Constructor[_] = thriftConstructorMemo(clazz)
    constructor.newInstance()
  }

  private def isPublic(clazz: Class[_], fieldId: TFieldIdEnum): Boolean = {
    val annotationMap: Map[TFieldIdEnum, Map[String, String]] = thriftFieldAnnotationsMemo(clazz)
    annotationMap
      .get(fieldId)
      .map(_.get(PublicFieldAnnotationKey).contains(PublicFieldAnnotationValue))
      .getOrElse(false)
  }
}
