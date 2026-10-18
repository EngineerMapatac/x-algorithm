package com.twitter.botmaker.runtime

import com.twitter.botmaker.rule.EventOutputCollector
import com.twitter.botmaker.rule.EventProcessor
import com.twitter.botmaker.rule.InputProvider
import com.twitter.botmaker.runtime.config.ScheduledEventConfigs
import com.twitter.botmaker.runtime.thriftjava.ScheduledEventConfig
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util._

case class ScheduleEventRegistry(
  config: ScheduledEventConfig,
  private val closing: Promise[Unit],
  private val task: Future[Unit])
    extends ServiceRegistry[ScheduledEventConfig] {

  override val name: String = config.eventType

  override def close(deadline: Time): Future[Unit] = {
    closing.setDone
    task
  }
}

trait ScheduledEventContainer extends ServiceContainer with EventProcessor {
  container =>

  override type _CONFIGS_TYPE <: ScheduledEventConfigs

  private def isReusable(config: ScheduledEventConfig) =
    isReusable[ScheduledEventConfig, ScheduleEventRegistry](config.eventType, config)

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {

    val scheduledEvents = configs.getScheduledEvents map {
      case config if isReusable(config) =>
        logger.info(s"reuse Timer Event ${config.eventType}")
        serviceRegistry[ScheduleEventRegistry](config.eventType).get

      case config =>
        logger.info(s"create Timer Event ${config.eventType}")

        val closing = new Promise[Unit]()
        val duration = Duration.fromMilliseconds(config.intervalMillis)

        def run(): Future[Unit] = {

          val triggers = Seq(
            Future.sleep(duration)(timer),
            closing
          )

          Future.select(triggers) flatMap {
            case _ if closing.isDefined =>
              Future.Unit
            case _ =>
              checkEvent(config.eventType, InputProvider.empty, new EventOutputCollector) rescue {
                case _ =>
                  Future.Unit
              } flatMap { _ => run() }
          }
        }

        val futureTask = run()
        ScheduleEventRegistry(config, closing, futureTask)
    }

    scheduledEvents ++ super.build(configs, scoped)
  }
}
