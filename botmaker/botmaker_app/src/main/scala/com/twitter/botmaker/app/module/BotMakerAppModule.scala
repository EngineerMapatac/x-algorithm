package com.twitter.botmaker.app.module

import com.google.inject.Provides
import com.google.inject.Singleton
import com.twitter.botmaker.common.RatelimitedLogger
import com.twitter.botmaker.loader.BotmakerRulesConfigbusLoader
import com.twitter.botmaker.loader.Loader
import com.twitter.botmaker.loader.Schedule
import com.twitter.botmaker.rule.thriftscala.BotMakerRulesPackage
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.runtime.RuntimeInfo
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.common.util.Clock
import com.twitter.configbus.client.file.PollingConfigSourceBuilder
import com.twitter.conversions.DurationOps._
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.inject.Injector
import com.twitter.inject.TwitterModule
import com.twitter.server.coordinate.ProcessPaths
import com.twitter.util.Await
import com.twitter.util.Duration
import com.twitter.util.ScheduledThreadPoolTimer
import com.twitter.app.Flag
import com.twitter.configbus.client.ConfigSource
import InjectionNamedConstants._

object BotMakerAppModule {
  val DEVEL: String = "devel"
  val LOCAL: String = "local"

  def createConfigbusBotMakerRulesPackageLoader(
    statsReceiver: StatsReceiver,
    configSource: ConfigSource,
    overrideRulesPackageFile: () => Option[String],
    defaultRulesPackageFile: () => String
  ): Loader[BotMakerRulesPackage] = {
    val rulesPackage = overrideRulesPackageFile().getOrElse(defaultRulesPackageFile())

    new BotmakerRulesConfigbusLoader(
      loaderName = "botmaker_configbus_loader",
      configSource = configSource,
      configFileName = rulesPackage,
      statsReceiver = statsReceiver
    )

  }
}

class BotMakerAppModule(val isAdmin: Boolean = false) extends TwitterModule {

  val cluster: Flag[String] =
    flag(BotMakerClusterFlag, BotMakerAppModule.LOCAL, "Data Center (smf1, atla, local)")
  val environment: Flag[String] =
    flag(BotMakerEnvironmentFlag, BotMakerAppModule.DEVEL, "(staging, prod, devel, etc)")
  val roleName: Flag[String] = flag(BotMakerRoleNameFlag, "local", "Role name")
  val shardId: Flag[Int] = flag(BotMakerShardIdFlag, 0, "Shard Id")
  val appServiceName: Flag[String] =
    flag(BotMakerAppServiceNameFlag, "botmaker-app-service", "Service name")
  val appName: Flag[String] = flag(BotMakerAppNameFlag, "botmaker-app", "BotMaker App name")
  val threads: Flag[Int] = flag(BotMakerThreadsFlag, 16, "Number of background threads")
  val refreshDelay: Flag[Int] = flag(BotMakerRefreshDelayFlag, 30, "Rules package refresh delay")

  val baseConfigPath: Flag[String] = flag[String](
    BotMakerBaseConfigPathFlag,
    "/usr/local/config",
    "Base config path"
  )

  val rulesPackageFile: Flag[String] = flag[String](
    BotMakerRulesPackageOverrideFileFlag,
    "",
    "Config file location"
  )

  val configSourcePollingInterval: Flag[Duration] = flag(
    BotMakerConfigSourcePollingIntervalFlag,
    30.seconds,
    "The interval for the service to check the config"
  )

  val botMakerRulesLoaderStartupTime: Flag[Duration] = flag(
    name = BotMakerRulesStartupTimeoutFlag,
    default = 30.seconds,
    help = "How long to wait for the BotMaker rules to be loaded as part of startup"
  )

  override def singletonStartup(injector: Injector): Unit = {
    super.singletonStartup(injector)
    val loader = injector.instance[Loader[BotMakerRulesPackage]]
    Await.result(loader.ready, botMakerRulesLoaderStartupTime())
  }

  @Provides
  @Singleton
  def provideClock(): Clock = {
    Clock.SYSTEM_CLOCK
  }

  @Provides
  @Singleton
  def provideTwitterRuntime(
    statsReceiver: StatsReceiver,
    serviceIdentifier: ServiceIdentifier,
    clock: Clock
  ): TwitterRuntime = {
    val logger = RatelimitedLogger.getLogger(appName())

    val processPath = ProcessPaths.fromEnv

    val info = new RuntimeInfo(
      cluster = cluster(),
      roleName = roleName(),
      environment = environment(),
      serviceName = appServiceName(),
      shardId = shardId(),
      isAdminInstance = isAdmin,
      serviceIdentifier = serviceIdentifier
    ) {
      override def debug(dbg: RuntimeDebugger): Unit = {

        super.debug(dbg)

        dbg.open("loader", this) foreach {
          _.infos(
            "baseConfigPath" -> baseConfigPath(),
            "defaultRulesPackageFile" -> defaultRulesPackageFile,
            "overrideRulesPackageFile" -> overrideRulesPackageFile.toString
          )
        }

        dbg.open("process", this) foreach {
          _.infos(
            "process" -> processPath.process.toString,
            "cluster" -> processPath.cluster.toString,
            "owner" -> processPath.owner.toString,
            "application" -> processPath.application.toString,
            "isFullyQualified" -> processPath.isFullyQualified.toString
          )
        }
      }
    }

    TwitterRuntime(
      name = appName(),
      clock = clock,
      statsReceiver = statsReceiver.scope("botmaker", appName()),
      timer = botmakerTimer,
      logger = logger,
      info = Some(info)
    )
  }

  @Provides
  @Singleton
  private def provideConfigSource(): ConfigSource = {
    info(s"creating ConfigSource with base path ${baseConfigPath()}...")
    PollingConfigSourceBuilder()
      .basePath(baseConfigPath())
      .pollPeriod(configSourcePollingInterval())
      .versionFilePath(None)
      .build()
  }

  private def botmakerTimer = new ScheduledThreadPoolTimer(
    threads(),
    "botmaker",
    false
  )

  private def defaultRulesPackageFile = {
    "botmaker/default/%s/%s/%s.json".format(
      environment(),
      appName(),
      appName()
    )
  }

  private def overrideRulesPackageFile: Option[String] = {
    if (rulesPackageFile().isEmpty) None else Some(rulesPackageFile())
  }

  @Singleton
  @Provides
  private def provideBotMakerRulesPackageLoader(
    statsReceiver: StatsReceiver,
    serviceIdentifier: ServiceIdentifier,
    configSource: ConfigSource
  ): Loader[BotMakerRulesPackage] = {
    createBotMakerRulesPackageLoader(statsReceiver, serviceIdentifier, configSource)
  }

  protected def createBotMakerRulesPackageLoader(
    statsReceiver: StatsReceiver,
    serviceIdentifier: ServiceIdentifier,
    configSource: ConfigSource
  ): Loader[BotMakerRulesPackage] = {
    BotMakerAppModule.createConfigbusBotMakerRulesPackageLoader(
      statsReceiver,
      configSource,
      () => overrideRulesPackageFile,
      () => defaultRulesPackageFile
    )
  }

  protected def createSchedule(
    loader: Loader[BotMakerRulesPackage]
  ): Schedule[BotMakerRulesPackage] = {
    Schedule(
      loader = loader,
      loadTimeout = Duration.fromSeconds(60),
      refreshDelay = 30.seconds
    )
  }
}
