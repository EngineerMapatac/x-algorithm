package com.twitter.botmaker.app.scarecrow

import com.google.inject.Provides
import com.google.inject.Singleton
import com.twitter.app.{Flag => CLFlag}
import com.twitter.botmaker.app.module.BotMakerApp
import com.twitter.botmaker.app.module.BotMakerAppModule
import com.twitter.botmaker.app.scarecrow.function.AdminFetcher
import com.twitter.botmaker.app.scarecrow.function.AdminRegistry
import com.twitter.botmaker.app.scarecrow.function.Fetcher20
import com.twitter.botmaker.app.scarecrow.function.NonAdminRegistry
import com.twitter.botmaker.app.scarecrow.legacy.ModelRuntime
import com.twitter.botmaker.fetcher.Fetcher10
import com.twitter.botmaker.legacyloader
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.util.logging.Logging
import com.twitter.util.Await
import com.twitter.util.Duration
import com.twitter.util.Timer
import java.util.concurrent.TimeUnit
import com.google.inject
import com.twitter.botmaker.app.module.InjectionNamedConstants
import com.twitter.botmaker.loader.Loader
import com.twitter.botmaker.rule.thriftscala.BotMakerRulesPackage
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.inject.annotations.Flag

class Module(isAdmin: Boolean) extends BotMakerAppModule(isAdmin) with Logging {

  val initializerTimeout: CLFlag[Int] =
    flag("botmaker.initializerTimeout", 5, "Botmaker initializer timeout (in minutes)")

  private val includeTranspiledBots =
    flag(
      "include_transpiled_bots",
      false,
      "Whether or not to include transpiled bots in compilation (transpiled DFs are always included)"
    )

  @Provides
  @Singleton
  def provideTimer(runtime: TwitterRuntime): Timer = {
    runtime.timer
  }

  @Provides
  @Singleton
  def provideScarecrowBotMaker(
    runtime: TwitterRuntime,
    fetcher10: Fetcher10,
    fetcher20: Fetcher20,
    adminFetcher: AdminFetcher,
    models: ModelRuntime,
    loader: Loader[BotMakerRulesPackage],
    @Flag(InjectionNamedConstants.BotMakerAppNameFlag) appName: String,
    @Flag(InjectionNamedConstants.BotMakerEnvironmentFlag) env: String
  ): ScarecrowBotMaker = {
    new ScarecrowBotMaker(
      runtime,
      fetcher10,
      fetcher20,
      adminFetcher,
      models,
      schedule = createSchedule(loader),
      appName,
      env,
      includeTranspiledBots()
    )
  }

  @Provides
  @Singleton
  def provideBotMakerApp(scarecrowBotMaker: ScarecrowBotMaker): BotMakerApp = {
    BotMakerApp(scarecrowBotMaker)
  }

  def provideProdGauge(statsReceiver: StatsReceiver): Unit = {
    statsReceiver.provideGauge(s"${this.getClass.getCanonicalName}.isAdmin") {
      {
        if (isAdmin) 1f else 0f
      }
    }
  }

  @Provides
  @Singleton
  def provideBotmakerInitializer(legacyScheduler: legacyloader.Scheduler): BotmakerInitializer = {
    new BotmakerInitializer {
      override def init(): Unit = {
        try {
          legacyScheduler.dump()
          val timeout = Duration(initializerTimeout(), TimeUnit.MINUTES)
          Await.result(legacyScheduler.ready, timeout)
          legacyScheduler.dump()
        } catch {
          case e: Exception =>
            logger.warn("failed to initialize legacy scheduler runtime", e)
            throw new RuntimeException("failed to initialize legacy scheduler runtime", e)
        }
      }
    }
  }

  override val modules: Seq[inject.Module] = {
    if (isAdmin) {
      super.modules ++ Seq(AdminRegistry)
    } else {
      super.modules ++ Seq(NonAdminRegistry)
    }
  }
}
