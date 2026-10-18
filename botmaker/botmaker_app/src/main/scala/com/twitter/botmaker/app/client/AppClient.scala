package com.twitter.botmaker.app.client

import java.util
import com.google.common.collect.ImmutableList
import com.google.common.collect.ImmutableMap

import scala.collection.JavaConverters._
import scala.reflect.ClassTag
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.app.client.config.AppClientConfigs
import com.twitter.botmaker.app.thriftjava.AppClientConfig
import com.twitter.botmaker.app.thriftjava.AppRequest
import com.twitter.botmaker.app.thriftjava.AppService
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.tuples.Pair
import com.twitter.botmaker.function.GeneralNode
import com.twitter.botmaker.runtime._
import com.twitter.botmaker.runtime.config._
import com.twitter.finagle.Addr
import com.twitter.finagle.Address
import com.twitter.finagle.Name
import com.twitter.finagle.Namer
import com.twitter.finagle.Resolver
import com.twitter.finagle.Service
import com.twitter.finagle.ThriftMux
import com.twitter.finagle.mtls.client.MtlsStackClient._
import com.twitter.finagle.partitioning.zk.ZkMetadata
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.finagle.thrift.ClientId
import com.twitter.finagle.thrift.RichClientParam
import com.twitter.finagle.thrift.ThriftClientRequest
import com.twitter.util.Duration
import com.twitter.util.Future
import com.twitter.util.Time
import com.twitter.util.Var
import org.apache.thrift.protocol.TBinaryProtocol

package config {

  object AppClient extends ConfigUnit[AppClientConfigs, AppClientConfig] {

    override def description: String =
      "Add a Botmaker App client. App client is used to send " +
        "events to other BotMaker Apps."

    override def apply(
      context: Context[AppClientConfigs],
      config: AppClientConfig
    ): AppClientConfig = {
      context.getRuntime.addAppClient(config)
      config
    }
  }

  trait AppClientConfigs extends ServiceConfigs {

    import scala.collection.mutable.ArrayBuffer

    final private val appClients = ArrayBuffer[AppClientConfig]()

    final def getAppClients: Seq[AppClientConfig] = appClients.toSeq

    final def addAppClient(config: AppClientConfig): Unit = {

      unique(Endpoint, config.endpoint)

      validateEndpoint(config.endpoint)

      validate(
        config.sessionTimeoutMillis >= 0,
        s"invalid session timeout value ${config.sessionTimeoutMillis} in app client ${config.endpoint}"
      )

      validate(
        config.requestTimeoutMillis >= 0,
        s"invalid request timeout value ${config.requestTimeoutMillis} in app client ${config.endpoint}"
      )

      validate(
        config.partitions.asScala.toSeq.forall(_ >= 0),
        s"invalid partitions ${config.partitions} in app client ${config.endpoint}"
      )

      validate(
        config.partitionFailovers >= 0,
        s"invalid partition failovers value ${config.partitionFailovers} in app client ${config.endpoint}"
      )

      appClients.append(config)
    }

    override def debug(dbg: RuntimeDebugger): Unit = {
      super.debug(dbg)

      dbg.open("appClients", appClients) foreach { d =>
        appClients foreach { ap => d.info(ap.endpoint, ap.toString) }
      }
    }

    override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
      val builder = super.compilerBuilder[E]

      appClients foreach { ac =>
        val p = com.twitter.botmaker.app.client.AppClient(ac)
        p.toCompile foreach { u => builder.addCompilerUnit(u) }
      }

      builder
    }
  }
}

case class AppClientEndpoint(
  client: AppService.ServiceIface,
  service: Service[ThriftClientRequest, Array[Byte]])

case class AppClientRegistry(
  config: AppClientConfig,
  endpoint: AppClientEndpoint,
  partitions: List[AppClientEndpoint])
    extends ServiceRegistry[AppClientConfig] {

  val requestType: Type = Type.mapOf(Type.STRING, Type.OBJECT)
  val responseType: Type = Type.mapOf(Type.STRING, Type.OBJECT)

  override val name: String = config.endpoint

  override def close(deadline: Time): Future[Unit] = {
    Future collect {
      endpoint.service.close(deadline) :: partitions.map(_.service.close(deadline))
    } unit
  }

  def checkEvent(
    eventType: String,
    inputFeatures: java.util.Map[String, AnyRef]
  ): Future[java.util.Map[String, AnyRef]] = {

    val inputData = requestType.serializer.serialize(inputFeatures, false)
    val request = new AppRequest(eventType, inputData)

    endpoint.client.botMakerAppCheckEvent(request).map { response =>
      responseType.serializer
        .deserialize(response.outputFeatures, false)
        .asInstanceOf[util.Map[String, AnyRef]]
    }
  }

  def checkEvent(
    eventType: String,
    index: Long,
    inputFeatures: java.util.Map[String, AnyRef]
  ): Future[java.util.Map[String, AnyRef]] = {

    val inputData = requestType.serializer.serialize(inputFeatures, false)
    val request = new AppRequest(eventType, inputData)

    val part = (index % partitions.size).toInt
    checkEvent(eventType, part, config.partitionFailovers.toInt, request)
  }

  def checkEvent(
    eventType: String,
    part: Int,
    tries: Int,
    request: AppRequest
  ): Future[java.util.Map[String, AnyRef]] = {

    partitions(part).client.botMakerAppCheckEvent(request).map { response =>
      responseType.serializer
        .deserialize(response.outputFeatures, false)
        .asInstanceOf[util.Map[String, AnyRef]]
    } rescue {
      case t: Throwable if tries > 0 =>
        val i = (part + 1) % partitions.size
        checkEvent(eventType, part, tries - 1, request)
      case t: Throwable =>
        Future.exception(t)
    }
  }

}

trait AppClientContainer extends ServiceContainer {
  self =>

  override type _CONFIGS_TYPE <: AppClientConfigs

  def isReusable(config: AppClientConfig): Boolean = {
    isReusable[AppClientConfig, AppClientRegistry](config.endpoint, config)
  }

  def appClients(endpoint: String): AppClientRegistry = {
    serviceRegistry[AppClientRegistry](endpoint).get
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val appClients = configs.getAppClients.map {
      case config if isReusable(config) =>
        logger.info(s"reuse AppClient ${config.endpoint}")
        serviceRegistry[AppClientRegistry](config.endpoint).get

      case config if config.isSetPartitions && config.partitions.size() > 0 =>
        logger.info(s"create AppClient ${config.endpoint} with partitions ${config.partitions}")

        val clientId: ClientId = ClientId(config.clientId)

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

        val ps = config.partitions.asScala.map(x => Option(Long.unbox(x).toInt)).toSet
        val addr = Namer.resolve(config.servicePath)

        val endpoint = build(
          clientId,
          partitionsName(addr, ps, (config.servicePath, config.partitions)),
          config.endpoint,
          sessionTimeout,
          requestTimeout,
          scoped.scope(config.endpoint)
        )

        val partitions = config.partitions.asScala.toList map { shard =>
          val servicePath = s"${config.servicePath}#$shard"
          logger.info(s"create AppClient ${config.endpoint} for $servicePath.")
          build(
            clientId,
            partitionsName(addr, Set(Some(Long.unbox(shard).toInt)), (config.servicePath, shard)),
            "",
            sessionTimeout,
            requestTimeout,
            scoped.scope(config.endpoint, shard.toString)
          )
        }

        AppClientRegistry(config, endpoint, partitions)

      case config =>
        logger.info(s"create AppClient ${config.endpoint}")

        val clientId: ClientId = ClientId(config.clientId)

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

        val endpoint = build(
          clientId,
          Resolver.eval(config.servicePath),
          config.endpoint,
          sessionTimeout,
          requestTimeout,
          scoped.scope(config.endpoint)
        )

        AppClientRegistry(config, endpoint, Nil)
    }

    appClients ++ super.build(configs, scoped)
  }

  private def partitionsName(addr: Var[Addr], partitionSet: Set[Option[Int]], id: Any) = {
    val partitionsAddr: Var[Addr] = addr map {
      case Addr.Bound(set, metadata) => {
        val filtered = set filter {
          case Address.Inet(ia, m)
              if m.contains("zk_metadata") &&
                m("zk_metadata").isInstanceOf[ZkMetadata] &&
                partitionSet.contains(m("zk_metadata").asInstanceOf[ZkMetadata].shardId) =>
            true
          case _ =>
            false
        }

        Addr.Bound(filtered, metadata)
      }

      case other: Addr =>
        other

      case _ =>
        Addr.Neg
    }

    Name.Bound(partitionsAddr, id)

  }

  private def build(
    clientId: ClientId,
    name: Name,
    label: String,
    sessionTimeout: Duration,
    requestTimeout: Duration,
    scoped: StatsReceiver
  ): AppClientEndpoint = {

    val thriftClient = ThriftMux.client
      .withMutualTls(serviceIdentifier)
      .withClientId(clientId)
      .withStatsReceiver(scoped)
      .withSession
      .acquisitionTimeout(sessionTimeout)
      .withRequestTimeout(requestTimeout)

    val service: Service[ThriftClientRequest, Array[Byte]] = thriftClient
      .newService(name, label)

    val iface: AppService.ServiceIface =
      thriftClient.build(
        name,
        label,
        classOf[AppService.ServiceIface],
        RichClientParam(protocolFactory = new TBinaryProtocol.Factory),
        service
      )
    AppClientEndpoint(iface, service)
  }
}

case class AppClient(config: AppClientConfig) {

  private val ns = config.endpoint + "."
  private val funcName = ns + "checkEvent"
  private val partName = ns + "checkPart"

  def toCompile: Seq[CompilerUnit] = {

    val checkFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.of(Type.STRING),
        Type.pairOf(Type.STRING, Type.OBJECT),
        Type.mapOf(Type.STRING, Type.OBJECT)
      )

      @Override
      def funcName = AppClient.this.funcName

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

        new GeneralNode[AppClientContainer](funcName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[AppClientContainer],
            args: util.List[AnyRef]
          ): Future[AnyRef] = {

            val runtime = context.getRuntime
            val client = runtime.appClients(config.endpoint)
            val eventType = args.get(0).asInstanceOf[String]

            val builder = ImmutableMap.builder[String, AnyRef]

            (1 until args.size()) foreach { i =>
              val pair = args.get(i).asInstanceOf[Pair[String, AnyRef]]
              builder.put(pair.getFirst, pair.getSecond)
            }

            val inputFeatures = builder.build
            client.checkEvent(eventType, inputFeatures)
          }
        }
      }
    }

    if (config.isSetPartitions && config.partitions.size > 0) {
      val partFunc = new CompilerUnit {

        private val signature = new Signature(
          ImmutableList.of(Type.STRING, Type.LONG),
          Type.pairOf(Type.STRING, Type.OBJECT),
          Type.mapOf(Type.STRING, Type.OBJECT)
        )

        @Override
        def funcName = AppClient.this.partName

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

          new GeneralNode[AppClientContainer](funcName, children) {

            override def getSignature = signature

            override protected def getCacheLevel = CacheLevel.Never

            override protected def computeClassName = funcName

            override protected def evaluate(
              context: Context[AppClientContainer],
              args: util.List[AnyRef]
            ): Future[AnyRef] = {

              val runtime = context.getRuntime
              val client = runtime.appClients(config.endpoint)
              val eventType = args.get(0).asInstanceOf[String]
              val part = args.get(1).asInstanceOf[java.lang.Long]

              val builder = ImmutableMap.builder[String, AnyRef]

              (2 until args.size()) foreach { i =>
                val pair = args.get(i).asInstanceOf[Pair[String, AnyRef]]
                builder.put(pair.getFirst, pair.getSecond)
              }

              val inputFeatures = builder.build
              client.checkEvent(eventType, Long.unbox(part), inputFeatures)
            }
          }
        }
      }
      List(checkFunc, partFunc)
    } else {
      List(checkFunc)
    }
  }
}
