package com.twitter.botmaker.runtime

import com.twitter.app.GlobalFlag
import org.apache.thrift.TBase
import org.apache.thrift.TFieldIdEnum
import com.twitter.botmaker.common.RatelimitedLogger
import com.twitter.common.util.Clock
import com.twitter.finagle.mtls.authentication.EmptyServiceIdentifier
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.stats.NullStatsReceiver
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util.Timer

object localMode
    extends GlobalFlag(
      false,
      "BotMaker is running locally without certain resources which may be on normal deployed " +
        "machines (ex: credentials files deployed via ConfigBus)") {

  def provideGauge(statsReceiver: StatsReceiver): Unit = {
    statsReceiver.provideGauge(this.getClass.getCanonicalName) {
      if (get.getOrElse(false)) 1f else 0f
    }
  }
}

object personalAccountMode
    extends GlobalFlag(
      false,
      "BotMaker is running as a personal account (as opposed to a service account). There are " +
        "some services (ex: Kafka) which require Kerberos keytabs which are not available when " +
        "running the application as a personal account rather than a service account.") {

  def provideGauge(statsReceiver: StatsReceiver): Unit = {
    statsReceiver.provideGauge(this.getClass.getCanonicalName) {
      if (get.getOrElse(false)) 1f else 0f
    }
  }
}

case class RuntimeInfo(
  cluster: String,
  roleName: String,
  environment: String,
  serviceName: String,
  shardId: Int,
  isAdminInstance: Boolean,
  serviceIdentifier: ServiceIdentifier = EmptyServiceIdentifier)
    extends Debuggable {

  def debug(dbg: RuntimeDebugger): Unit = dbg.infos(
    "cluster" -> cluster,
    "roleName" -> roleName,
    "environment" -> environment,
    "serviceName" -> serviceName,
    "shardId" -> shardId.toString,
    "isAdminInstance" -> isAdminInstance.toString,
    "serviceIdentifier" -> serviceIdentifier.toString
  )
}

trait TwitterRuntime extends Runtime with Debuggable {

  def name: String
  def clock: Clock
  def statsReceiver: StatsReceiver
  def timer: Timer
  def logger: RatelimitedLogger

  def info: Option[RuntimeInfo]

  def cacheGet(key: AnyRef)(f: => AnyRef): AnyRef = f

  def debug(dbg: RuntimeDebugger): Unit = {

    dbg.open("runtime", this) foreach { d =>
      d.infos(
        "name" -> name,
        "className" -> this.getClass.getName,
        "TypeName" -> this.getClass.getTypeName
      )

      info foreach { i => d.open("info", i) foreach { i.debug } }
    }
  }

  def serviceIdentifier: ServiceIdentifier = {
    info.map(_.serviceIdentifier).getOrElse(EmptyServiceIdentifier)
  }
}

object TwitterRuntime {
  type Thrift[T <: TBase[T, F], F <: TFieldIdEnum] = TBase[T, F]

  def apply(
    name: String = "BotMaker",
    clock: Clock = Clock.SYSTEM_CLOCK,
    statsReceiver: StatsReceiver = NullStatsReceiver,
    timer: Timer = Timer.Nil
  ): TwitterRuntime = {

    val logger = RatelimitedLogger.getLogger(name)

    this.apply(
      name = name,
      clock = clock,
      statsReceiver = statsReceiver,
      timer = timer,
      logger = logger,
      info = None
    )
  }

  def apply(
    name: String,
    clock: Clock,
    statsReceiver: StatsReceiver,
    timer: Timer,
    logger: RatelimitedLogger,
    info: Option[RuntimeInfo]
  ): TwitterRuntime = {

    val aName = name
    val aClock = clock
    val aStatsReceiver = statsReceiver
    val aTimer = timer
    val aLogger = logger
    val aInfo = info

    new TwitterRuntime {
      override def name: String = aName

      override def clock: Clock = aClock

      override def statsReceiver: StatsReceiver = aStatsReceiver

      override def timer: Timer = aTimer

      override def logger: RatelimitedLogger = aLogger

      override def info: Option[RuntimeInfo] = aInfo
    }
  }
}
