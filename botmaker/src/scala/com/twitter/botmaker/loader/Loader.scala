package com.twitter.botmaker.loader

import com.twitter.finagle.stats.Counter
import com.twitter.finagle.stats.Stat
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util.Future
import com.twitter.util.TimeoutException
import java.util.concurrent.atomic.AtomicBoolean
import org.slf4j.Logger
import org.slf4j.LoggerFactory

trait Loader[T] {

  val name: String

  def load(): Future[T]

  def ready: Future[Unit] = Future.Done
}

abstract class AbstractLoader[T](
  override val name: String,
  initialValue: Option[T],
  statsReceiver: StatsReceiver)
    extends Loader[T] {

  private[this] val logger: Logger = LoggerFactory.getLogger(getClass)

  private[this] val scoped: StatsReceiver = statsReceiver.scope(name)

  private[this] val successCounter: Counter = scoped.counter("success")
  private[this] val failureCounter: Counter = scoped.counter("failure")
  private[this] val timeoutCounter: Counter = scoped.counter("timeout")
  private[this] val latencyStat: Stat = scoped.stat("latency")

  private[this] val initialized = new AtomicBoolean(false)

  def this(name: String, statsReceiver: StatsReceiver) {
    this(name, None, statsReceiver)
  }

  def this(name: String, initialValue: T, statsReceiver: StatsReceiver) {
    this(name, Some(initialValue), statsReceiver)
  }

  protected def fetch(): Future[T]

  final def load(): Future[T] =
    Stat.timeFuture(latencyStat) {
      if (!initialized.get() && initialValue.isDefined && initialized.compareAndSet(false, true)) {
        Future.value(initialValue.get)
      } else {
        Future.Unit flatMap { _ =>
          logger.info(s"Loading $name ...")
          fetch()
        } onSuccess { _ =>
          logger.info(s"Success $name")
          successCounter.incr()
        } onFailure {
          case t: TimeoutException =>
            logger.warn(s"$name load timeout:", t)
            timeoutCounter.incr()
            failureCounter.incr()
          case t: Throwable =>
            logger.warn(s"$name load failed:", t)
            failureCounter.incr()
        }
      }
    }
}
