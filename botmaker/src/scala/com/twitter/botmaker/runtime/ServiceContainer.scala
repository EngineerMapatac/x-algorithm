package com.twitter.botmaker.runtime

import scala.reflect.ClassTag
import com.twitter.botmaker.common.RatelimitedLogger
import com.twitter.botmaker.runtime.config.ServiceConfigs
import com.twitter.common.util.Clock
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util.Closable
import com.twitter.util.Timer

trait ServiceRegistry[+RC] extends Closable with Debuggable {
  def name: String
  def config: RC

  def isReusable[RC1 >: RC](c: RC1): Boolean = {
    config == c
  }

  def debug(dbg: RuntimeDebugger): Unit = {
    dbg.info("config", config.toString)
  }
}

trait ServiceContainer extends TwitterRuntime {

  type _CONFIGS_TYPE <: ServiceConfigs

  def runtime: TwitterRuntime

  final override def name: String = runtime.name

  final override def clock: Clock = runtime.clock

  final override def statsReceiver: StatsReceiver = runtime.statsReceiver

  final override def timer: Timer = runtime.timer

  final override def logger: RatelimitedLogger = runtime.logger

  final override def info: Option[RuntimeInfo] = runtime.info

  def serviceRegistry[T <: ServiceRegistry[_]](name: String)(implicit ctag: ClassTag[T]): Option[T]

  def isReusable[CT, T <: ServiceRegistry[CT]: ClassTag](name: String, config: CT): Boolean =
    serviceRegistry[T](name) exists {
      _.isReusable(config)
    }

  def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = Nil
}

trait ServiceRuntime[CT <: ServiceConfigs] extends ServiceContainer {

  override type _CONFIGS_TYPE = CT
}
