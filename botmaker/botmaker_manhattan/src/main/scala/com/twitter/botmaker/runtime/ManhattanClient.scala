package com.twitter.botmaker.runtime

import java.util
import com.google.common.collect.ImmutableList
import com.twitter.bijection.Injection
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.tuples.Tuple
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.types.TypeCompiler
import com.twitter.botmaker.compiler.types.TypeContext
import com.twitter.botmaker.function.GeneralNode
import com.twitter.botmaker.runtime.thriftjava.ManhattanClientConfig
import com.twitter.botmaker.runtime.thriftjava.ThriftProtocol
import com.twitter.io.Buf
import com.twitter.stitch.Stitch
import com.twitter.storage.client.manhattan.bijections.Bijections.BufInjection
import com.twitter.storage.client.manhattan.bijections.Bijections.LongInjection
import com.twitter.storage.client.manhattan.bijections.Bijections.StringInjection
import com.twitter.storage.client.manhattan.kv.impl.KeyDescriptor
import com.twitter.storage.client.manhattan.kv.impl.ValueDescriptor
import com.twitter.storage.client.manhattan.kv.ManhattanKVEndpoint
import com.twitter.storage.client.manhattan.kv.ManhattanKey
import com.twitter.storage.client.manhattan.kv.ManhattanKeyRange
import com.twitter.storage.client.manhattan.kv.ManhattanKeyRangeEndpoint
import com.twitter.storage.client.manhattan.kv.ManhattanValue
import com.twitter.util.Duration
import com.twitter.util.Future
import org.apache.thrift.protocol.TBinaryProtocol
import org.apache.thrift.protocol.TCompactProtocol
import org.apache.thrift.protocol.TProtocolFactory

import scala.collection.JavaConverters
import scala.collection.JavaConverters._

object ManhattanClient {

  private def toBuf(
    tp: Type,
    va: AnyRef,
    allowNulls: Boolean,
    tProtocolFactory: TProtocolFactory
  ): Buf = (tp, va) match {
    case (t, v: String) if t == Type.STRING =>
      StringInjection(v)
    case (t, v: java.lang.Long) if t == Type.LONG =>
      LongInjection(v)
    case (t, v) =>
      BufInjection(t.serializer.serialize(v, allowNulls, tProtocolFactory))
  }

  private def fromBuf(
    tp: Type,
    buf: Buf,
    allowNulls: Boolean,
    tProtocolFactory: TProtocolFactory
  ): AnyRef = tp match {
    case Type.STRING =>
      StringInjection.invert(buf).get
    case Type.LONG =>
      Long.box(LongInjection.invert(buf).get)
    case t =>
      val bytebuf = BufInjection.invert(buf)
      t.serializer.deserialize(bytebuf.get, allowNulls, tProtocolFactory)
  }

  private case class FullKey(
    dataset: String,
    pkeyTypes: Seq[Type],
    lkeyTypes: Seq[Type],
    pkeys: Seq[AnyRef],
    lkeys: Seq[AnyRef],
    allowNulls: Boolean,
    tProtocolFactory: TProtocolFactory)
      extends KeyDescriptor.FullKey[FullKey] {

    override def apply(): ManhattanKey = ManhattanKey(
      dataset = dataset,
      pkey = pkeyTypes zip pkeys map {
        case (t, k) => toBuf(t, k, allowNulls, tProtocolFactory)
      },
      lkey = lkeyTypes zip lkeys map {
        case (t, k) => toBuf(t, k, allowNulls, tProtocolFactory)
      }
    )

    override def invert(
      ds: String,
      pkey: Seq[Buf],
      lkey: Seq[Buf]
    ): FullKey = FullKey(
      dataset,
      pkeyTypes,
      lkeyTypes,
      pkeyTypes zip pkey map {
        case (t, b) => fromBuf(t, b, allowNulls, tProtocolFactory)
      },
      lkeyTypes zip lkey map {
        case (t, b) => fromBuf(t, b, allowNulls, tProtocolFactory)
      },
      allowNulls,
      tProtocolFactory
    )
  }

  private case class FullRangeKey(
    dataset: String,
    pkeyTypes: Seq[Type],
    lkeyTypes: Seq[Type],
    pkeys: Seq[AnyRef],
    lkeys: Seq[AnyRef],
    allowNulls: Boolean,
    tProtocolFactory: TProtocolFactory)
      extends KeyDescriptor.FullRangeKey[FullKey] {

    override def apply(): ManhattanKeyRange = {
      val pkeyBuf = pkeyTypes zip pkeys map {
        case (t, k) => toBuf(t, k, allowNulls, tProtocolFactory)
      }
      val lkeyBuf = lkeyTypes.take(lkeys.size) zip lkeys map {
        case (t, k) => toBuf(t, k, allowNulls, tProtocolFactory)
      }

      ManhattanKeyRange(
        ManhattanKeyRangeEndpoint(
          ManhattanKey(dataset, pkeyBuf, lkeyBuf),
          negativeInfinity = true,
          partInclusive = false
        ),
        ManhattanKeyRangeEndpoint(
          ManhattanKey(dataset, pkeyBuf, lkeyBuf),
          positiveInfinity = true,
          partInclusive = false
        )
      )
    }

    override def mkKey(ds: String, pkey: Seq[Buf], lkey: Seq[Buf]): FullKey = {
      val pkeyValues = (pkeyTypes zip pkey) map {
        case (t, k) => fromBuf(t, k, allowNulls, tProtocolFactory)
      }
      val lkeyValues = (lkeyTypes zip lkey) map {
        case (t, k) => fromBuf(t, k, allowNulls, tProtocolFactory)
      }
      FullKey(dataset, pkeyTypes, lkeyTypes, pkeyValues, lkeyValues, allowNulls, tProtocolFactory)
    }
  }

  def insert(
    endpoint: ManhattanKVEndpoint,
    dataset: String,
    pkeyTypes: Seq[Type],
    lkeyTypes: Seq[Type],
    valueType: Type,
    pkeys: Seq[AnyRef],
    lkeys: Seq[AnyRef],
    value: AnyRef,
    ttl: Option[Duration],
    allowNulls: Boolean,
    tProtocolFactory: TProtocolFactory
  ): Future[Unit] = {
    val keyDesc = FullKey(
      dataset,
      pkeyTypes,
      lkeyTypes,
      pkeys,
      lkeys,
      allowNulls,
      tProtocolFactory
    )

    val buf = toBuf(valueType, value, allowNulls, tProtocolFactory)
    val mhValue = ManhattanValue(contents = buf, ttl = ttl)
    val valueDesc = ValueDescriptor.FullValue(Injection.identity[Buf], mhValue)
    val stitch = endpoint.insert(keyDesc, valueDesc)
    Stitch.run(stitch)
  }

  def get(
    endpoint: ManhattanKVEndpoint,
    dataset: String,
    pkeyTypes: Seq[Type],
    lkeyTypes: Seq[Type],
    valueType: Type,
    pkeys: Seq[AnyRef],
    lkeys: Seq[AnyRef],
    allowNulls: Boolean,
    tProtocolFactory: TProtocolFactory
  ): Future[Option[AnyRef]] = {
    val keyDesc = FullKey(
      dataset,
      pkeyTypes,
      lkeyTypes,
      pkeys,
      lkeys,
      allowNulls,
      tProtocolFactory
    )
    val valueDesc = ValueDescriptor(Injection.identity[Buf])

    val stitch = endpoint.get(keyDesc, valueDesc) map { optVal =>
      optVal map { value => fromBuf(valueType, value.contents, allowNulls, tProtocolFactory) }
    }

    Stitch.run(stitch)
  }

  def getAll(
    endpoint: ManhattanKVEndpoint,
    dataset: String,
    pkeyTypes: Seq[Type],
    lkeyTypes: Seq[Type],
    valueType: Type,
    pkeys: Seq[AnyRef],
    allowNulls: Boolean,
    limit: Option[Int],
    tProtocolFactory: TProtocolFactory
  ): Future[Seq[Tuple]] = {
    val keyDesc = FullRangeKey(
      dataset,
      pkeyTypes,
      lkeyTypes,
      pkeys,
      Nil,
      allowNulls,
      tProtocolFactory
    )
    val valueDesc = ValueDescriptor(Injection.identity[Buf])

    val stitch = endpoint.slice(keyDesc, valueDesc, limit) map { seq =>
      seq map {
        case (k, v) =>
          Tuple.of(
            (k.lkeys :+ fromBuf(valueType, v.contents, allowNulls, tProtocolFactory)).toArray)
      }
    }

    Stitch.run(stitch)
  }

}

case class ManhattanClientOperator(config: ManhattanClientConfig) {

  private val ns = config.endpoint + "."
  private val setFuncName = ns + "Set" + config.funcName
  private val getFuncName = ns + "Get" + config.funcName
  private val getAllFuncName = ns + "GetAll" + config.funcName

  val name: String = config.funcName

  val ttl: Option[Duration] = if (config.isSetTtlMillis) {
    Some(Duration.fromMilliseconds(config.ttlMillis))
  } else {
    None
  }

  val allowNulls: Boolean = config.allowNulls

  private val protocolFactory: TProtocolFactory = config.thriftProtocol match {
    case ThriftProtocol.COMPACT => new TCompactProtocol.Factory
    case ThriftProtocol.BINARY => new TBinaryProtocol.Factory
  }

  def toCompile(typeContext: TypeContext): List[CompilerUnit] = {

    val keyTypes = config.keyTypes.asScala map { a => TypeCompiler.getType(a, typeContext) } toList

    val localKeyTypes = config.localKeyTypes.asScala map { a =>
      TypeCompiler.getType(a, typeContext)
    } toList

    val valueType = TypeCompiler.getType(config.valueType, typeContext)

    val setFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf(
          JavaConverters.asJavaIterable((keyTypes ::: localKeyTypes) :+ valueType)),
        Type.UNIT
      )

      @Override
      def funcName = setFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"set value by key in Manhattan dataset ${config.dataset}. ${config.description}",
        ActionLevel.PROD_OTHER_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @Override
      @throws(classOf[SemanticCheckFailure])
      def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[ManhattanEndpointContainer](setFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[ManhattanEndpointContainer],
            args: util.List[AnyRef]
          ): Future[AnyRef] = {
            val endpoint = context.getRuntime.manhattanEndpoints(config.endpoint)
            ManhattanClient
              .insert(
                endpoint = endpoint,
                dataset = config.dataset,
                pkeyTypes = keyTypes,
                lkeyTypes = localKeyTypes,
                valueType = valueType,
                pkeys = args.asScala,
                lkeys = args.asScala.drop(keyTypes.size),
                value = args.get(args.size - 1),
                ttl = ttl,
                allowNulls = allowNulls,
                tProtocolFactory = protocolFactory
              )
              .voided
          }
        }
      }
    }

    val getFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf(JavaConverters.asJavaIterable(keyTypes ::: localKeyTypes)),
        ImmutableList.of(valueType),
        valueType
      )

      @Override
      def funcName = getFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"get value by key in Manhattan dataset ${config.dataset}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @Override
      @throws(classOf[SemanticCheckFailure])
      def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[ManhattanEndpointContainer](getFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[ManhattanEndpointContainer],
            args: util.List[AnyRef]
          ): Future[AnyRef] = {
            val endpoint = context.getRuntime.manhattanEndpoints(config.endpoint)
            ManhattanClient.get(
              endpoint = endpoint,
              dataset = config.dataset,
              pkeyTypes = keyTypes,
              lkeyTypes = localKeyTypes,
              valueType = valueType,
              pkeys = args.asScala,
              lkeys = args.asScala.drop(keyTypes.size),
              allowNulls = allowNulls,
              tProtocolFactory = protocolFactory
            ) map {
              case Some(value) =>
                value
              case None if (args.size > keyTypes.size + localKeyTypes.size) =>
                args.get(keyTypes.size + localKeyTypes.size)
              case None =>
                null
            }
          }
        }
      }
    }

    val getAllFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf(JavaConverters.asJavaIterable(keyTypes)),
        ImmutableList.of(Type.LONG),
        Type.listOf(Type.tupleOf(localKeyTypes :+ valueType: _*))
      )

      @Override
      def funcName = getAllFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"get all lkeys and values in Manhattan dataset ${config.dataset}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @Override
      @throws(classOf[SemanticCheckFailure])
      def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[ManhattanEndpointContainer](getAllFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[ManhattanEndpointContainer],
            args: util.List[AnyRef]
          ): Future[AnyRef] = {

            val limit = args.asScala.drop(keyTypes.size).headOption.map(_.asInstanceOf[Long].toInt)
            val endpoint = context.getRuntime.manhattanEndpoints(config.endpoint)
            ManhattanClient.getAll(
              endpoint = endpoint,
              dataset = config.dataset,
              pkeyTypes = keyTypes,
              lkeyTypes = localKeyTypes,
              valueType = valueType,
              pkeys = args.asScala,
              allowNulls = allowNulls,
              limit,
              tProtocolFactory = protocolFactory
            ) map {
              _.asJava
            }
          }
        }
      }
    }

    if (localKeyTypes.size == 0) {
      List(setFunc, getFunc)
    } else {
      List(setFunc, getFunc, getAllFunc)
    }
  }
}
