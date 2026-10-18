package com.twitter.botmaker.runtime

import java.util
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerDoc
import com.twitter.botmaker.compiler.CompilerUnit
import com.twitter.botmaker.compiler.Fingerprint
import com.twitter.botmaker.function.GeneralNode
import com.twitter.botmaker.runtime.config.HttpEndpointConfigs
import com.twitter.botmaker.runtime.thriftjava.HttpEndpointConfig
import com.twitter.config.yaml.YamlMap
import com.twitter.finagle.Http
import com.twitter.finagle.Service
import com.twitter.finagle.http.Method
import com.twitter.finagle.http.ProxyCredentials
import com.twitter.finagle.http.Request
import com.twitter.finagle.http.RequestBuilder
import com.twitter.finagle.http.Response
import com.twitter.finagle.service.FailFastFactory.FailFast
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.io.Buf
import com.twitter.io.Bufs
import com.twitter.util.Duration
import com.twitter.util.Future
import com.twitter.util.Time

case class HttpEndpointRegistry(
  config: HttpEndpointConfig,
  credentials: Option[ProxyCredentials],
  service: Service[Request, Response])
    extends ServiceRegistry[HttpEndpointConfig] {

  override val name: String = config.endpoint

  override def close(deadline: Time): Future[Unit] = service.close(deadline)
}

trait HttpEndpointFactory extends TwitterRuntime {

  private def getProxyCredentials(config: HttpEndpointConfig): Option[ProxyCredentials] = {
    if (config.isSetProxyCredentialPath
      && config.isSetProxyUserName
      && config.isSetProxyUserPassword) {
      val yaml: YamlMap = YamlMap.load(config.proxyCredentialPath)
      for {
        user <- yaml.getString(config.proxyUserName)
        password <- yaml.getString(config.proxyUserPassword)
      } yield {
        ProxyCredentials(user, password)
      }
    } else {
      None
    }
  }

  def build(
    config: HttpEndpointConfig,
    scoped: StatsReceiver
  ): (Option[ProxyCredentials], Service[Request, Response]) = {
    val hostPort: String = s"${config.host}:${config.port}"
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
    val client: Http.Client = if (config.withTls) {
      Http.client.withTls(config.host)
    } else {
      Http.client
    }
    val service: Service[Request, Response] = client
      .configured(FailFast(enabled = false))
      .withSession
      .acquisitionTimeout(sessionTimeout)
      .withRequestTimeout(requestTimeout)
      .withStatsReceiver(scoped.scope(config.endpoint))
      .newService(hostPort)

    val credentials: Option[ProxyCredentials] = getProxyCredentials(config)
    (credentials, service)
  }
}

trait HttpEndpointContainer extends ServiceContainer with HttpEndpointFactory {
  self =>
  override type _CONFIGS_TYPE <: HttpEndpointConfigs

  private def isReusable(config: HttpEndpointConfig) =
    isReusable[HttpEndpointConfig, HttpEndpointRegistry](config.endpoint, config)

  def httpServices(config: HttpEndpointConfig): HttpEndpointRegistry = {
    serviceRegistry[HttpEndpointRegistry](config.endpoint).get
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val httpEndpoints = configs.getHttpEndpoints.map {
      case config if isReusable(config) =>
        logger.info(s"reuse HttpClient ${config.endpoint}")
        serviceRegistry[HttpEndpointRegistry](config.endpoint).get
      case config =>
        logger.info(s"create HttpClient ${config.endpoint}")
        val (credentials, service) = build(config, scoped.scope("HttpEndpoint"))
        HttpEndpointRegistry(config, credentials, service)
    }
    httpEndpoints ++ super.build(configs, scoped)
  }
}

case class HttpClientOperator(config: HttpEndpointConfig) {

  private val functionName = config.endpoint + "." + "Http"
  private val signature = new Signature(
    ImmutableList.of(Type.STRING, Type.STRING, Type.mapOf(Type.STRING, Type.STRING)),
    ImmutableList.of(Type.STRING),
    Type.pairOf(Type.LONG, Type.STRING)
  )

  def toCompile: CompilerUnit = new CompilerUnit {
    override def funcName(): String = functionName

    override def botMakerDoc(): BotMakerDoc = BotMakerDoc.of(
      functionName,
      signature,
      config.description,
      ActionLevel.PROD_OTHER_ACTION
    )

    override def fingerprint(): Fingerprint = Fingerprint.of(config)

    @throws(classOf[SemanticCheckFailure])
    override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
      new GeneralNode[HttpEndpointContainer](
        functionName,
        children
      ) {
        override def getSignature: ASTNode.Signature = signature

        override protected def getCacheLevel = CacheLevel.Never

        override protected def computeClassName = functionName

        override protected def evaluate(
          context: Context[HttpEndpointContainer],
          args: util.List[AnyRef]
        ): Future[AnyRef] = {
          import scala.collection.JavaConverters._

          val registry: HttpEndpointRegistry = context.getRuntime.httpServices(config)
          val method: Method = Method(args.get(0).asInstanceOf[String])
          val url: String = args.get(1).asInstanceOf[String]
          val headers: Map[String, String] =
            args.get(2).asInstanceOf[java.util.Map[String, String]].asScala.toMap
          val content: Option[Buf] =
            if (args.size() == 4) {
              Some(Bufs.UTF_8.apply(args.get(3).asInstanceOf[String]))
            } else {
              None
            }
          val requestBuilder: RequestBuilder[Nothing, Nothing] =
            if (registry.credentials.isEmpty) {
              RequestBuilder()
            } else {
              RequestBuilder().proxied(registry.credentials)
            }

          val request = requestBuilder
            .url(url)
            .addHeaders(headers)
            .build(method, content)

          registry.service.apply(request).map { rep: Response =>
            (rep.status.code, Bufs.asUtf8String(rep.content))
          }
        }
      }
    }
  }
}
