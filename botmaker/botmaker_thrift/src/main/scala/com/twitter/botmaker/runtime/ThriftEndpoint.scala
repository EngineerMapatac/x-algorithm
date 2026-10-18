package com.twitter.botmaker.runtime

import java.util
import scala.reflect.ClassTag
import scala.collection.JavaConverters._
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.CacheLevel
import org.apache.thrift.TApplicationException
import org.apache.thrift.TException
import org.apache.thrift.protocol.TBinaryProtocol
import org.apache.thrift.protocol.TMessage
import org.apache.thrift.protocol.TMessageType
import org.apache.thrift.protocol.TProtocolFactory
import org.apache.thrift.transport.TMemoryInputTransport
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.ThriftType
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.GeneralNode
import com.twitter.botmaker.function.thrift.ThriftOperator
import com.twitter.botmaker.runtime.TwitterRuntime.Thrift
import com.twitter.botmaker.runtime.config.ThriftEndpointConfigs
import com.twitter.botmaker.runtime.thriftjava.ThriftEndpointConfig
import com.twitter.botmaker.runtime.thriftjava.ThriftMethodConfig
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.finagle.mtls.client.MtlsStackClient._
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.finagle.thrift.ClientId
import com.twitter.finagle.thrift.RichClientParam
import com.twitter.finagle.thrift.ThriftClientRequest
import com.twitter.finagle.Name
import com.twitter.finagle.Resolver
import com.twitter.finagle.Service
import com.twitter.finagle.Thrift
import com.twitter.finagle.ThriftMux
import com.twitter.scrooge.TReusableBuffer
import com.twitter.util.Duration
import com.twitter.util.Future
import com.twitter.util.Time

case class ThriftEndpointRegistry(
  config: ThriftEndpointConfig,
  client: AnyRef,
  service: Service[ThriftClientRequest, Array[Byte]])
    extends ServiceRegistry[ThriftEndpointConfig] {

  override val name: String = config.endpoint

  override def close(deadline: Time): Future[Unit] = service.close(deadline)
}

trait ThriftEndpointFactory extends TwitterRuntime {

  def build(
    config: ThriftEndpointConfig,
    scoped: StatsReceiver
  ): (AnyRef, Service[ThriftClientRequest, Array[Byte]]) = {

    val clientId: ClientId = ClientId(config.clientId)
    val (name: Name, label: String) = Resolver.evalLabeled(config.servicePath)

    val requestTimeout: Duration = if (config.isSetRequestTimeoutMillis) {
      Duration.fromMilliseconds(config.requestTimeoutMillis)
    } else {
      Duration.Top
    }

    val sessionTimeout: Duration = if (config.isSetSessionTimeoutMillis) {
      Duration.fromMilliseconds(config.sessionTimeoutMillis)
    } else {
      Duration.Top
    }

    val clazz: Class[_] = ClassCache.forName(config.thriftClass)

    config.thriftClass match {
      case "org.apache.scribe.scribe$ServiceIface" =>
        val thriftClient = Thrift.client
          .withClientId(clientId)
          .withStatsReceiver(scoped.scope(config.endpoint))
          .withProtocolFactory(ThriftMethod.protocolFactory)
          .withRequestTimeout(requestTimeout)
        val service: Service[ThriftClientRequest, Array[Byte]] = thriftClient
          .newService(name, label)
        val clientIface: AnyRef =
          thriftClient.build(
            name,
            label,
            clazz,
            RichClientParam(protocolFactory = ThriftMethod.protocolFactory),
            service
          )

        (clientIface, service)

      case _ =>
        val thriftClient = ThriftMux.client
          .withMutualTls(serviceIdentifier)
          .withClientId(clientId)
          .withStatsReceiver(scoped.scope(config.endpoint))
          .withSession
          .acquisitionTimeout(sessionTimeout)
          .withRequestTimeout(requestTimeout)

        val service: Service[ThriftClientRequest, Array[Byte]] = thriftClient
          .newService(name, label)
        val clientIface: AnyRef =
          thriftClient.build(
            name,
            label,
            clazz,
            RichClientParam(protocolFactory = ThriftMethod.protocolFactory),
            service
          )

        (clientIface, service)
    }
  }
}

trait ThriftEndpointContainer extends ServiceContainer with ThriftEndpointFactory {
  self =>

  override type _CONFIGS_TYPE <: ThriftEndpointConfigs

  def isReusable(config: ThriftEndpointConfig): Boolean = {
    isReusable[ThriftEndpointConfig, ThriftEndpointRegistry](config.endpoint, config)
  }

  def thriftEndpoints[Iface: ClassTag](endpoint: String): Iface = {
    serviceRegistry[ThriftEndpointRegistry](endpoint).get.client.asInstanceOf[Iface]
  }

  def thriftServices(endpoint: String): Service[ThriftClientRequest, Array[Byte]] = {
    serviceRegistry[ThriftEndpointRegistry](endpoint).get.service
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val thriftEndpoints = configs.getThriftEndpoints.map {
      case config if isReusable(config) =>
        logger.info(s"reuse ThriftEndpoint ${config.endpoint}")
        serviceRegistry[ThriftEndpointRegistry](config.endpoint).get
      case config =>
        logger.info(s"create ThriftEndpoint ${config.endpoint}")
        val (client, service) = build(config, scoped.scope("ThriftEndpoint"))
        ThriftEndpointRegistry(config, client, service)
    }
    thriftEndpoints ++ super.build(configs, scoped)
  }
}

object ThriftMethod {

  import java.util.Arrays

  val tlReusableBuffer: TReusableBuffer = TReusableBuffer()
  val protocolFactory: TProtocolFactory = new TBinaryProtocol.Factory

  private val seqId: Int = 0

  def parse(method: String, resultType: ThriftType, bytes: Array[Byte]): Thrift[_, _] = {
    val trans = new TMemoryInputTransport(bytes)
    val iprot = protocolFactory.getProtocol(trans)
    val msg = iprot.readMessageBegin
    if (msg.`type` == TMessageType.EXCEPTION) {
      val x = TApplicationException.readFrom(iprot)
      iprot.readMessageEnd()
      throw x
    }
    if (msg.seqid != seqId) {
      throw new TApplicationException(
        TApplicationException.BAD_SEQUENCE_ID,
        s"$method failed: out of sequence response"
      )
    }

    val result = resultType.newInstance
    result.read(iprot)
    iprot.readMessageEnd()
    result.asInstanceOf[Thrift[_, _]]
  }

  def call(
    runtime: TwitterRuntime,
    service: Service[ThriftClientRequest, Array[Byte]],
    method: String,
    request: Thrift[_, _],
    timeout: Duration,
    resultType: ThriftType
  ): Future[AnyRef] =
    try {
      val transport = tlReusableBuffer.take()
      val protocol = protocolFactory.getProtocol(transport)
      protocol.writeMessageBegin(new TMessage(method, TMessageType.CALL, 0))
      request.write(protocol)
      protocol.writeMessageEnd()

      val buffer = Arrays.copyOfRange(transport.getArray, 0, transport.length)
      val thriftRequest: ThriftClientRequest = new ThriftClientRequest(buffer, false)

      service
        .apply(thriftRequest)
        .raiseWithin(timeout)(runtime.timer) flatMap { bytes =>
        try {
          val result = parse(method, resultType, bytes)
          val value = resultType.getFieldNames.asScala map { fieldName =>
            resultType.getFieldValue(result, fieldName)
          } filter { value => value != null } headOption

          value match {
            case Some(t: Throwable) => Future.exception(t)
            case Some(x: AnyRef) => Future.value(x)
            case None => Future.Void
          }
        } catch {
          case e: Exception =>
            Future.exception(e)
        }
      } onFailure {
        case t: Throwable =>
      }
    } catch {
      case e: TException =>
        Future.exception(e)
    } finally {
      tlReusableBuffer.reset()
    }
}

case class ThriftMethod(thriftEndpoint: ThriftEndpointConfig, config: ThriftMethodConfig) {

  private val serviceClass = ClassCache
    .forName(thriftEndpoint.thriftClass)

  private val method = serviceClass.getMethods.toList.filter(_.getName == config.methodName).head

  private val declaringClass = {
    val name = method.getDeclaringClass.getName
    name.substring(0, name.lastIndexOf("$"))
  }

  private val argsClass = {
    val name = declaringClass + "$" + config.methodName + "_args"
    ClassCache.forName(name).asInstanceOf[Class[TwitterRuntime.Thrift[_, _]]]
  }

  private val argsType = Type.thriftOf(argsClass)

  private val resultClass = {
    val name = declaringClass + "$" + config.methodName + "_result"
    ClassCache.forName(name).asInstanceOf[Class[TwitterRuntime.Thrift[_, _]]]
  }

  private val resultType = Type.thriftOf(resultClass)

  private val returnType = if (resultType.getFieldNames.contains("success")) {
    resultType.getFieldType("success")
  } else {
    Type.UNIT
  }

  private val timeout = if (config.isSetTimeoutMillis) {
    Duration.fromMilliseconds(config.timeoutMillis)
  } else {
    Duration.Top
  }

  private val TPA = Type.newGenericType
  private val TPB = Type.newGenericType
  private val argSignature = new ASTNode.Signature(
    Type.pairOf(TPA, TPB),
    argsType
  )

  private val ns = thriftEndpoint.endpoint + "."
  private val funcName = ns + config.funcName
  private val signature = new ASTNode.Signature(
    ImmutableList.of[Type](Type.thriftOf(argsClass)),
    returnType
  )

  def name: String = config.getFuncName

  class ArgsNode(exprText: String, children: ImmutableList[ASTNode[_]])
      extends ThriftOperator[ThriftEndpointContainer, TwitterRuntime.Thrift[_, _]](
        argsClass,
        exprText,
        children
      ) {
    override def getSignature: ASTNode.Signature = argSignature

    override protected def apply(
      context: Context[ThriftEndpointContainer],
      thriftObject: Thrift[_, _]
    ): AnyRef = {
      thriftObject
    }
  }

  def toCompile: CompilerUnit = new CompilerUnit {

    @Override
    def funcName = ThriftMethod.this.funcName

    override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
      funcName,
      signature,
      config.description,
      ActionLevel.PROD_OTHER_ACTION
    )

    override def fingerprint(): Fingerprint = Fingerprint.of(config)

    @Override
    @throws(classOf[SemanticCheckFailure])
    def createASTNode(children: ImmutableList[ASTNode[_]]) = {
      val argsChildren = ImmutableList.of[ASTNode[_]](new ArgsNode("args", children))
      new GeneralNode[ThriftEndpointContainer](funcName, argsChildren) {

        override def getSignature = signature

        override protected def getCacheLevel = CacheLevel.Never

        override protected def computeClassName = funcName

        override protected def evaluate(
          context: Context[ThriftEndpointContainer],
          args: util.List[AnyRef]
        ): Future[AnyRef] = {
          val service = context.getRuntime.thriftServices(thriftEndpoint.endpoint)
          val request = args.get(0).asInstanceOf[TwitterRuntime.Thrift[_, _]]
          ThriftMethod
            .call(context.getRuntime, service, config.methodName, request, timeout, resultType)
        }
      }
    }
  }
}
