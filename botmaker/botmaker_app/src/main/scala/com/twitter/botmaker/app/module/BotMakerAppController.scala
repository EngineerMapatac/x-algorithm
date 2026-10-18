package com.twitter.botmaker.app.module

import BotMakerAppController.addTracing
import BotMakerAppController.objectMapper
import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.scala.ScalaObjectMapper
import com.google.common.base.Strings
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.BotMakerEvent
import com.twitter.botmaker.BotMakerResponse
import com.twitter.botmaker.app.thriftjava._
import com.twitter.botmaker.common.RatelimitedLogger
import com.twitter.botmaker.common.SerializerUtil
import com.twitter.botmaker.compiler.BotMakerDoc
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.rule._
import com.twitter.botmaker.rule.function.ResponseCodes
import com.twitter.botmaker.rule.thriftjava.{BotMakerRulesPackage => JBotMakerRulesPackage}
import com.twitter.botmaker.rule.thriftscala.{BotMakerRulesPackage => SBotMakerRulesPackage}
import com.twitter.finagle.tracing.Annotation.BinaryAnnotation
import com.twitter.finagle.tracing.Trace
import com.twitter.service.gen.scarecrow.TieredAction
import com.twitter.util.Future
import com.twitter.util.logging.Logging
import java.util.{Map => JMap}
import scala.collection.JavaConverters._

abstract class BotMakerAppController extends Logging {

  def botmaker: BotMakerApp

  val log: RatelimitedLogger = RatelimitedLogger.getLogger(s"${botmaker.name}.thrift")

  def botMakerAppCheckEvent(request: AppRequest): Future[AppResponse] = {
    val inputFeatures = BotMakerAppController.requestType.serializer
      .deserialize(request.inputFeature, false)
      .asInstanceOf[java.util.Map[String, AnyRef]]

    addTracing(request.eventType)

    botmaker.checkEvent(
      request.eventType,
      InputProvider(inputFeatures),
      new EventOutputCollector with EventOutput
    ) map { output =>
      val outputFeatures = output.outputFeatures
      new AppResponse()
        .setOutputFeatures(
          BotMakerAppController.responseType.serializer.serialize(outputFeatures, false)
        )
    } onFailure { t: Throwable =>
      log.warning("botMakerAppCheckEvent", s"botMakerAppCheckEvent(${request.eventType})", t)
    }
  }

  def botMakerCheckEvent(event: BotMakerEvent): Future[TieredAction] = {

    addTracing(event.eventType)

    botmaker.checkAppEvent(event.eventType)(
      InputProvider.transform(event.inputFeatures)
    ) map { output =>
      val response = new BotMakerResponse
      response.setRuleResponses(output.allRuleResponses)
      response.setFinalResponse(output.finalRuleResponse)
      response.setOutputFeatures(output.outputFeatureData)

      val tieredAction = ResponseCodes.toTiredAction(event.eventType, response)
      tieredAction
    } onFailure { t: Throwable =>
      log.warning("botMakerCheckEvent", s"botMakerCheckEvent(${event.eventType})", t)
    }
  }

  def botMakerRunExpression(request: RunExprRequest): Future[RunExprResponse] = {

    Future.Unit flatMap { _ =>
      if (Strings.isNullOrEmpty(request.inputFeatures)) {
        Future.value(InputProvider.empty)
      } else {
        botmaker.runExpression(request.eventType, request.inputFeatures)(InputProvider.empty).map {
          case (_, result) =>
            result.asInstanceOf[JMap[String, AnyRef]].asScala
        }
      }
    } flatMap { inputFeatures =>
      addTracing(request.eventType)
      botmaker.runExpression(request.eventType, request.expression)(inputFeatures)
    } map {
      case (_, result: RunExprResponse) =>
        info(message = s"botMakerRunExpression($request) returns $result")
        result
      case (_, result) =>
        info(message = s"botMakerRunExpression($request) returns $result")
        new RunExprResponse()
          .setResult(s"$result")
          .setDisplay("text")
    } onFailure { t: Throwable =>
      log.warning(s"botMakerRunExpression($request) failed:", t)
    }
  }

  def botMakerRunExprWithFeatureData(
    request: RunExprWithFeatureDataRequest
  ): Future[RunExprWithFeatureDataResponse] = {
    Future.Unit flatMap { _ =>
      addTracing(request.eventType)
      botmaker
        .runExpression(request.eventType, request.expression) {
          InputProvider.transform(request.inputFeatures)
        }
    } map {
      case (_, result: RunExprResponse) =>
        info(message = s"botMakerRunExprWithFeatureData($request) returns $result")
        new RunExprWithFeatureDataResponse()
          .setResult(result.result)
          .setDisplay(result.display)
      case (_, result) =>
        info(s"botMakerRunExprWithFeatureData($request) returns $result")
        new RunExprWithFeatureDataResponse()
          .setResult(s"$result")
          .setDisplay("text")
    } onFailure { t: Throwable =>
      log.warning(s"botMakerRunExpression($request) failed:", t)
    }
  }

  def botMakerCheckPackage(request: CheckPackageRequest): Future[CheckPackageResponse] = {
    Future.Unit flatMap { _ =>
      val rulesPackage = SerializerUtil.convert[JBotMakerRulesPackage, SBotMakerRulesPackage](
        SBotMakerRulesPackage,
        request.rulesPackage
      )
      botmaker.checkPackage(rulesPackage)
    } map { _ =>
      val response = new CheckPackageResponse(true)
      log.info(
        "botMakerCheckPackage",
        s"botMakerCheckPackage returns $response"
      )
      response
    } onFailure { t: Throwable => log.warning("botMakerCheckPackage", s"botMakerCheckPackage", t) }
  }

  def botMakerCheckSyntax(request: CheckSyntaxRequest): Future[CheckSyntaxResponse] = {
    Future {
      botmaker.checkSyntax(
        request.eventType,
        request.expression,
        if (!Strings.isNullOrEmpty(request.returnType)) request.returnType else "N/A",
        request.isDerivedFeature
      )
    } map { _ =>
      new CheckSyntaxResponse(true)
    } onFailure { t: Throwable =>
      log.warning("botMakerCheckSyntax", s"botMakerCheckSyntax($request)", t)
    }
  }

  def botMakerGetBotMakerDoc(request: getBotMakerDocRequest): Future[getBotMakerDocResponse] = {
    Future {
      botmaker.getBotMakerDoc(request.category)
    } map { results: Seq[BotMakerDoc] =>
      val entries = results.map { result =>
        new BotMakerDocEntry()
          .setName(result.name)
          .setDescription(result.description)
          .setIsDerivedFeature(result.isDerivedFeature)
          .setArgTypes(ImmutableList.copyOf(result.argTypes))
          .setArguments(ImmutableList.copyOf(result.arguments))
          .setReturnType(result.returnType)
          .setActionLevelName(result.actionLevel.name)
      }
      new getBotMakerDocResponse()
        .setEntries(entries.asJava)
    } onFailure { t: Throwable =>
      log.warning("botMakerGetBotMakerDoc", s"botMakerGetBotMakerDoc($request)", t)
    }
  }

  def botMakerGetAppInfo(): Future[getAppInfoResponse] = {
    Future {
      new getAppInfoResponse(botmaker.getAppInfo("*").asJava)
    } onFailure { t: Throwable => log.warning("botMakerGetAppInfo", "botMakerGetAppInfo", t) }
  }

  def botMakerGetEventTypes(): Future[getEventTypesResponse] = {
    Future {
      botmaker.getEventTypes()
    } map { results =>
      new getEventTypesResponse()
        .setInfos(results.asJava)
    } onFailure { t: Throwable => log.warning("botMakerGetEventTypes", "botMakerGetEventTypes", t) }
  }

  def checkJsonEvent(request: JsonRequest): Future[JsonResponse] = {

    val apiCalls = objectMapper.readValue[JMap[String, JMap[String, AnyRef]]](request.inputJson)
    val eventType: String = request.eventType

    val futures: Seq[Future[(String, JMap[String, AnyRef])]] =
      apiCalls.asScala.toSeq.map {
        case (funcName, params) =>
          val inputFeatures: InputProvider =
            params.asScala + ("api" -> funcName)
          botmaker
            .checkLocalEvent(eventType)(inputFeatures)
            .map { output: LocalEventOutputCollector =>
              funcName -> output.outputFeatures
            }
      }
    Future.collect(futures).map { responses: Seq[(String, JMap[String, AnyRef])] =>
      val responsesAsMap = responses.toMap.asJava
      new JsonResponse(objectMapper.writeValueAsString(responsesAsMap))
    }
  }

  def getTeams(): Future[GetTeamsResponse] = {
    Future {
      botmaker.getTeams()
    } map { result =>
      new GetTeamsResponse()
        .setTeams(result.asJava)
    } onFailure { t: Throwable => log.warning("getTeams", "getTeams", t) }
  }

  def testSimpleListValues(request: SimpleListTestRequest): Future[SimpleListTestResponse] = {
    Future.const(botmaker.testSimpleListValues(request))
  }

  def analyzeCode(): Future[AnalysisResponse] = {
    botmaker.analyzeCode()
  }
}

object BotMakerAppController {
  private val requestType: Type = Type.mapOf(Type.STRING, Type.OBJECT)
  private val responseType: Type = Type.mapOf(Type.STRING, Type.OBJECT)

  private val objectMapper: ObjectMapper with ScalaObjectMapper = {
    val mapper = new ObjectMapper with ScalaObjectMapper
    mapper.configure(DeserializationFeature.USE_LONG_FOR_INTS, true)
    mapper
  }

  private def addTracing(eventType: String): Unit = {
    if (Trace.isActivelyTracing) {
      Trace.record(BinaryAnnotation("botmakerEventType", eventType))
    }
  }
}
