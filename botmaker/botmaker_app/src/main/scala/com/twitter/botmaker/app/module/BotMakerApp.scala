package com.twitter.botmaker.app.module

import com.twitter.botmaker.app.thriftjava.AnalysisPerTargetResponse
import com.twitter.botmaker.app.thriftjava.AnalysisResponse
import com.twitter.botmaker.app.thriftjava.AnalysisTarget
import com.twitter.botmaker.app.thriftjava.SimpleListTestRequest
import com.twitter.botmaker.app.thriftjava.SimpleListTestResponse
import com.twitter.botmaker.compiler.BotMakerDoc
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.rule.thriftscala.BotMakerRulesPackage
import com.twitter.botmaker.rule.EventOutputCollector
import com.twitter.botmaker.rule.InputProvider
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.thriftjava.BotMakerEventTypeConfig
import com.twitter.botmaker.runtime.thriftjava.TeamConfig
import com.twitter.util.Closable
import com.twitter.util.Future
import com.twitter.util.Time
import com.twitter.util.Try
import scala.collection.JavaConverters._

trait BotMakerApp extends Closable {

  def name: String

  def runExpression(
    eventType: String,
    expr: String
  )(
    inputFeature: InputProvider
  ): Future[(Type, Any)]

  def checkSyntax(
    category: String,
    expr: String,
    returnType: String,
    isDerivedFeature: Boolean
  ): Unit

  def checkPackage(rulesPackage: BotMakerRulesPackage): Future[Unit]

  def getAppInfo(path: String*): Map[String, String]

  def getBotMakerDoc(category: String): Seq[BotMakerDoc]

  def getEventTypes(): Map[String, BotMakerEventTypeConfig]

  def getTeams(): Seq[TeamConfig]

  def testSimpleListValues(request: SimpleListTestRequest): Try[SimpleListTestResponse]

  def analyzeCode(): Future[AnalysisResponse]

  def checkEvent[T <: EventOutputCollector](
    eventType: String,
    inputFeatures: InputProvider,
    outputCollector: T
  ): Future[T]

  def checkAppEvent(
    eventType: String
  )(
    inputFeatures: InputProvider
  ): Future[AppEventOutputCollector]

  def checkLocalEvent(
    eventType: String
  )(
    inputFeatures: InputProvider
  ): Future[LocalEventOutputCollector]
}

object BotMakerApp {

  def apply(botmaker: AppBotMaker[_, _]): BotMakerApp = new BotMakerApp {

    def name: String = botmaker.runtime.name

    def close(deadline: Time): Future[Unit] = {
      botmaker.close(deadline)
    }

    def runExpression(
      eventType: String,
      expr: String
    )(
      inputFeature: InputProvider
    ): Future[(Type, Any)] = {
      botmaker.runExpression(eventType, expr)(inputFeature)
    }

    def checkSyntax(
      eventType: String,
      expr: String,
      returnType: String,
      isDerivedFeature: Boolean
    ): Unit = {
      botmaker.checkSyntax(eventType, expr, returnType, isDerivedFeature)
    }

    def checkPackage(rulesPackage: BotMakerRulesPackage): Future[Unit] = {
      botmaker.checkPackage(rulesPackage)
    }

    def getBotMakerDoc(category: String): Seq[BotMakerDoc] = {
      botmaker.state.getBotMakerDoc(category)
    }

    def getAppInfo(path: String*): Map[String, String] = {
      RuntimeDebugger.infos(botmaker.runtime, path: _*)
    }

    def getEventTypes(): Map[String, BotMakerEventTypeConfig] = {
      botmaker.state.getEventTypes()
    }

    def getTeams(): Seq[TeamConfig] = {
      botmaker.state.getTeams()
    }

    override def testSimpleListValues(
      request: SimpleListTestRequest
    ): Try[SimpleListTestResponse] = {
      botmaker.testSimpleListValues(request)
    }

    override def analyzeCode(): Future[AnalysisResponse] = Future {
      val theMap = botmaker.state.analyzeRuleCode().map {
        case (botId, resultPerBotMap) =>
          val perTargetResponse = new AnalysisPerTargetResponse()
            .setResult(resultPerBotMap.asJava)
          (AnalysisTarget.ruleId(botId), perTargetResponse)
      }
      new AnalysisResponse().setTargetResponseMap(theMap.asJava)
    }

    override def checkEvent[T <: EventOutputCollector](
      eventType: String,
      inputFeatures: InputProvider,
      outputCollector: T
    ): Future[T] = {
      botmaker.checkEvent(eventType, inputFeatures, outputCollector)
    }

    override def checkAppEvent(eventType: String)(inputFeatures: InputProvider) = {
      botmaker.checkAppEvent(eventType, inputFeatures)
    }

    override def checkLocalEvent(eventType: String)(inputFeatures: InputProvider) = {
      botmaker.checkLocalEvent(eventType, inputFeatures)
    }
  }
}
