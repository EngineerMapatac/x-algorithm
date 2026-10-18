package com.twitter.botmaker.app.server

import com.twitter.botmaker.app.module.BotMakerApp
import com.twitter.botmaker.app.thriftjava.AppService
import com.twitter.botmaker.rule.BotMaker
import com.twitter.botmaker.rule.BotMakerRuntime
import com.twitter.botmaker.rule.config.SystemConfigs
import com.twitter.finatra.mtls.thriftmux.AbstractMtlsThriftServer
import com.twitter.finatra.mtls.thriftmux.modules.MtlsThriftWebFormsModule
import com.twitter.finatra.thrift.filters.LoggingMDCFilter
import com.twitter.finatra.thrift.filters.ThriftMDCFilter
import com.twitter.finatra.thrift.filters.TraceIdMDCFilter
import com.twitter.finatra.thrift.routing.JavaThriftRouter
import com.twitter.inject.modules.StatsReceiverModule
import com.twitter.inject.thrift.modules.ThriftClientIdModule
import com.twitter.inject.TwitterModule
import com.twitter.thriftwebforms.MethodOptions
import com.twitter.util.Await
import com.twitter.util.Duration
import com.twitter.util.Futures
import java.util.concurrent.TimeUnit

class BotMakerAppServer extends AbstractMtlsThriftServer {
  override def name = "BotMakerApp"

  protected def warmupTimeout: Duration = Duration(3, TimeUnit.MINUTES)

  override def modules: Seq[TwitterModule] = Seq(
    ThriftClientIdModule,
    StatsReceiverModule,
    new MtlsThriftWebFormsModule[AppService.ServiceIface](this) {
      override def defaultMethodAccess: MethodOptions.Access = thriftWebFormsAccess
    }
  )

  override def defaultCloseGracePeriod: Duration = Duration(5, TimeUnit.SECONDS)

  protected def thriftWebFormsAccess: MethodOptions.Access = MethodOptions.Access.None

  override def configureThrift(router: JavaThriftRouter): Unit = {
    router
      .filter(classOf[LoggingMDCFilter])
      .filter(classOf[TraceIdMDCFilter])
      .filter(classOf[ThriftMDCFilter])
      .add(classOf[AppThriftController])
  }

  protected def postWarmup[CT <: SystemConfigs, RT <: BotMakerRuntime[CT]](
    botmaker: BotMaker[CT, RT]
  ): Unit = {
    try {
      val scheduler = botmaker.scheduler
      scheduler.dump()

      val allFutures =
        Futures.join(scheduler.ready, botmaker.ready, botmaker.success)
      Await.result(allFutures, warmupTimeout)

      scheduler.dump()
    } catch {
      case e: Exception =>
        logger.warn("failed to initialize botmaker runtime", e)
        throw new RuntimeException("failed to initialize botmaker runtime", e)
    }
  }

  override protected def postWarmup(): Unit = {
    logger.info(s"Post Warming up $name app")

    val botMakerApp = injector.instance[BotMakerApp]
    closeOnExit(botMakerApp)

    super.postWarmup()
  }

  override protected def start(): Unit = {
    logger.info(s"Start $name app")
  }

}
