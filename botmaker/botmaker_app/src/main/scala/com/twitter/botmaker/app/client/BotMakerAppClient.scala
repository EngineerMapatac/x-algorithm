package com.twitter.botmaker.app.client

import com.google.common.collect.ImmutableMap
import com.twitter.botmaker.app.thriftscala._
import com.twitter.botmaker.compiler.exceptions.RuntimeFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.rule.thriftscala.BotMakerRulesPackage
import com.twitter.botmaker.runtime.thriftscala.BotMakerEventTypeConfig
import com.twitter.botmaker.runtime.thriftscala.TeamConfig
import com.twitter.botmaker.thriftscala.BotMakerEvent
import com.twitter.botmaker.thriftscala.BotMakerResponse
import com.twitter.finagle.ThriftMux
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.mtls.client.MtlsStackClient._
import com.twitter.finagle.param.Stats
import com.twitter.finagle.stats.NullStatsReceiver
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.finagle.thrift.ClientId
import com.twitter.relevance.feature_store.thriftscala.FeatureData
import com.twitter.util.Duration
import com.twitter.util.Future
import java.util.{Map => JMap}

trait BotMakerAppClient {

  def checkSyntax(
    eventType: Option[String],
    expr: String,
    returnType: Option[String],
    isDerivedFeature: Option[Boolean]
  ): Future[Unit]

  def appCheckEvent(
    eventType: String,
    inputFeatures: JMap[String, AnyRef]
  ): Future[JMap[String, AnyRef]]

  def checkEvent(
    eventType: String,
    inputFeatures: Map[String, FeatureData]
  ): Future[BotMakerResponse]

  def checkJsonEvent(eventType: String, inputJson: String): Future[String]

  def runExpression(
    expr: String,
    eventTypeOpt: Option[String],
    inputFeatures: Option[String]
  ): Future[(String, String)]

  def runExprWithFeatureData(
    expr: String,
    eventTypeOpt: Option[String],
    inputFeatures: Option[Map[String, FeatureData]]
  ): Future[(String, String)]

  def checkBotMakerPackage(rulesPackage: BotMakerRulesPackage): Future[Unit]

  def getBotMakerDocEntry(category: String): Future[Seq[BotMakerDocEntry]]

  def getAppInfo(): Future[scala.collection.Map[String, String]]

  def getBotMakerEventTypes(): Future[scala.collection.Map[String, BotMakerEventTypeConfig]]

  def getTeams(): Future[Seq[TeamConfig]]

  def getBotCodeAnalysis(
  ): Future[scala.collection.Map[Long, scala.collection.Map[String, FeatureData]]]
}

object BotMakerAppClient {
  val clientId: ClientId = ClientId("botmakerapp-client")
  val statsRoot = "BotMakerAppClient"

  def apply(
    dest: String,
    serviceIdentifier: ServiceIdentifier,
    statsReceiver: StatsReceiver = NullStatsReceiver
  ): BotMakerAppClient = {
    apply(mux(dest, serviceIdentifier))
  }

  def staging(
    dest: String,
    serviceIdentifier: ServiceIdentifier,
    dtabOverride: String = "/srv=>/srv#/staging"
  ): BotMakerAppClient = {
    com.twitter.server.Init()
    com.twitter.finagle.Dtab.base = com.twitter.finagle.Dtab.read(dtabOverride)

    apply(mux(dest, serviceIdentifier))
  }

  def prod(
    dest: String,
    serviceIdentifier: ServiceIdentifier,
    dtabOverride: String = "/srv=>/srv#/prod"
  ): BotMakerAppClient = {
    com.twitter.server.Init()
    com.twitter.finagle.Dtab.base = com.twitter.finagle.Dtab.read(dtabOverride)

    apply(mux(dest, serviceIdentifier))
  }

  def mux(
    dest: String,
    serviceIdentifier: ServiceIdentifier,
    statsReceiver: StatsReceiver = NullStatsReceiver
  ): AppService.MethodPerEndpoint = {

    ThriftMux.client
      .withMutualTls(serviceIdentifier)
      .withClientId(clientId)
      .withSession
      .acquisitionTimeout(Duration.fromSeconds(5))
      .withRequestTimeout(Duration.fromSeconds(600))
      .configured(Stats(statsReceiver.scope(statsRoot)))
      .build[AppService.MethodPerEndpoint](dest)
  }

  def apply(client: AppService.MethodPerEndpoint): BotMakerAppClient = new BotMakerAppClient {

    override def checkSyntax(
      eventType: Option[String],
      expr: String,
      returnType: Option[String],
      isDerivedFeature: Option[Boolean]
    ): Future[Unit] = {
      client
        .botMakerCheckSyntax(CheckSyntaxRequest(eventType, expr, returnType, isDerivedFeature))
        .unit
    }

    override def appCheckEvent(
      eventType: String,
      inputFeatures: JMap[String, AnyRef]
    ): Future[JMap[String, AnyRef]] = {
      val requestType = Type.mapOf(Type.STRING, Type.OBJECT)
      val responseType = Type.mapOf(Type.STRING, Type.OBJECT)

      val dataBuf = requestType.serializer
        .serialize(inputFeatures, false)

      client
        .botMakerAppCheckEvent(AppRequest(eventType, dataBuf))
        .map { result =>
          val outputFeatures = responseType.serializer
            .deserialize(result.outputFeatures, false)
            .asInstanceOf[JMap[String, AnyRef]]

          ImmutableMap.of("outputFeatures", outputFeatures)
        }
    }

    override def checkEvent(
      eventType: String,
      inputFeatures: Map[String, FeatureData]
    ): Future[BotMakerResponse] = {

      client.botMakerCheckEvent(
        BotMakerEvent(Some(eventType), inputFeatures)
      ) map { tiedAction =>
        tiedAction.botMakerResponse match {
          case Some(response) => response
          case None => throw new RuntimeFailure("corrupted response data.")
        }
      }
    }

    override def checkJsonEvent(eventType: String, inputJson: String): Future[String] = {
      client
        .checkJsonEvent(JsonRequest(eventType = eventType, inputJson = inputJson))
        .map(_.outputJson)
    }

    override def runExpression(
      expr: String,
      eventTypeOpt: Option[String],
      inputFeatures: Option[String]
    ): Future[(String, String)] = {
      val runExprRequest = RunExprRequest(expr, eventTypeOpt, inputFeatures)
      client.botMakerRunExpression(runExprRequest) map { r => r.display -> r.result }
    }

    override def runExprWithFeatureData(
      expr: String,
      eventTypeOpt: Option[String],
      inputFeatures: Option[Map[String, FeatureData]]
    ): Future[(String, String)] = {
      val request = RunExprWithFeatureDataRequest(expr, eventTypeOpt, inputFeatures)
      client.botMakerRunExprWithFeatureData(request) map { r => r.display -> r.result }
    }

    override def checkBotMakerPackage(rulesPackage: BotMakerRulesPackage): Future[Unit] = {
      client
        .botMakerCheckPackage(CheckPackageRequest(rulesPackage))
        .unit
    }

    override def getBotMakerDocEntry(category: String): Future[Seq[BotMakerDocEntry]] = {
      client
        .botMakerGetBotMakerDoc(GetBotMakerDocRequest(category))
        .map(_.entries)
    }

    override def getAppInfo(): Future[scala.collection.Map[String, String]] = {
      client
        .botMakerGetAppInfo()
        .map(_.infos)
    }

    override def getBotMakerEventTypes(
    ): Future[scala.collection.Map[String, BotMakerEventTypeConfig]] = {
      client
        .botMakerGetEventTypes()
        .map(_.infos)
    }

    override def getTeams(): Future[Seq[TeamConfig]] = {
      client
        .getTeams()
        .map(_.teams)
    }

    override def getBotCodeAnalysis(
    ): Future[scala.collection.Map[Long, scala.collection.Map[String, FeatureData]]] = {
      client
        .analyzeCode()
        .map { response =>
          response.targetResponseMap.collect {
            case (AnalysisTarget.RuleId(id), AnalysisPerTargetResponse(resultMap)) =>
              (id, resultMap)
          }
        }
    }
  }
}
