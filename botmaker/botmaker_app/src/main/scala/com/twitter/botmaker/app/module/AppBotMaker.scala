package com.twitter.botmaker.app.module

import com.twitter.botmaker.app.thriftjava.SimpleListTestRequest
import com.twitter.botmaker.app.thriftjava.SimpleListTestResponse
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.loader.Schedule
import com.twitter.botmaker.rule.config.SystemConfigs
import com.twitter.botmaker.rule.thriftscala.BotMakerRulesPackage
import com.twitter.botmaker.rule._
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.finagle.FailureFlags
import com.twitter.finagle.mux.ClientDiscardedRequestException
import com.twitter.util.Future
import com.twitter.util.Throw
import com.twitter.util.Try

class AppEventOutputCollector
    extends EventOutputCollector
    with ResponseOutput
    with EventOutputFeatureData

class LocalEventOutputCollector extends EventOutputCollector with EventOutput

abstract class AppBotMaker[CT <: SystemConfigs, RT <: BotMakerRuntime[CT]](
  runtime: TwitterRuntime,
  schedule: Schedule[BotMakerRulesPackage],
  filterRules: BotMakerRulesPackage => BotMakerRulesPackage = identity
)(
  implicit ct: Manifest[CT],
  rt: Manifest[RT])
    extends BotMaker[CT, RT](runtime, schedule, filterRules) {

  def packageNames: Seq[String] = {
    List(this.getClass.getPackage.getName + ".function")
  }

  def configPackageNames: Seq[String] = {
    List(this.getClass.getPackage.getName + ".config")
  }

  override protected def configCompilerBuilder: CompilerBuilder[CT] = {
    val builder = super.configCompilerBuilder
    configPackageNames.foreach(builder.addPackage)
    builder
  }

  override protected def systemCompilerBuilder(systemConfigs: CT): CompilerBuilder[RT] = {
    val builder = super.systemCompilerBuilder(systemConfigs)
    packageNames.foreach(builder.addPackage)
    builder
  }

  override protected def compilerBuilder(systemConfigs: CT): CompilerBuilder[RT] = {
    val builder = super.compilerBuilder(systemConfigs)

    packageNames.foreach(builder.addPackage)
    builder
  }

  override protected def classifyException: Throwable => ExceptionCategory = {
    case cdre: ClientDiscardedRequestException if cdre.isFlagged(FailureFlags.Ignorable) =>
      Ignorable
    case cdre: ClientDiscardedRequestException if cdre.isFlagged(FailureFlags.Interrupted) =>
      Interruption
    case _ => NormalException
  }

  def checkAppEvent(
    eventType: String,
    inputFeatures: InputProvider
  ): Future[AppEventOutputCollector] = {
    checkEvent(eventType, inputFeatures, new AppEventOutputCollector)
  }

  def checkLocalEvent(
    eventType: String,
    inputFeatures: InputProvider
  ): Future[LocalEventOutputCollector] = {
    checkEvent(eventType, inputFeatures, new LocalEventOutputCollector)
  }

  def testSimpleListValues(request: SimpleListTestRequest): Try[SimpleListTestResponse] = {
    Throw(new RuntimeException("Can't test simple list values on this BotMaker app"))
  }
}
