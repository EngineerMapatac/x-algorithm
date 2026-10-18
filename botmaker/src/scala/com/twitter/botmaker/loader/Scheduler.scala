package com.twitter.botmaker.loader

import com.twitter.concurrent.AsyncSemaphore
import com.twitter.util._
import java.util.concurrent.atomic.AtomicBoolean
import java.util.logging.Logger

case class Schedule[T](
  loader: Loader[T],
  var loadTimeout: Duration,
  var retries: Seq[Int] = Nil,
  var refreshDelay: Duration,
  var retryDelay: Duration = Duration.Zero,
  required: Boolean = false) {

  def retry(r: Int): Schedule[T] = copy(retries = Seq(r, r))

  def retry(r0: Int, r1: Int): Schedule[T] = copy(retries = Seq(r0, r1))
}

class Scheduler(val timer: Timer) extends Closable {

  val logger: Logger = Logger.getLogger(classOf[Scheduler].getName)

  lazy private val semaphore = new AsyncSemaphore(maxConcurrent)

  private var schedules = Map[String, ScheduledItem[_]]()

  private val closed = new AtomicBoolean(false)

  val maxConcurrent: Int = Int.MaxValue

  private case class ScheduledItem[T](var schedule: Schedule[T]) extends Closable {

    val ready: Promise[ReadWriteVar[T]] = new Promise[ReadWriteVar[T]]()

    private val Seq(retry0, retry1) = (schedule.retries ++ Seq(0, 0)).take(2)

    private def start(retries: Int): Future[Unit] = {

      if (closed.get) {
        Future.Unit
      } else {
        semaphore
          .acquireAndRun {
            schedule.loader.load()
          }
          .raiseWithin(schedule.loadTimeout)(timer)
          .transform {
            case _ if closed.get =>
              Future.Unit
            case r: Return[T] if ready.isDefined =>
              val value = r()
              Try {
                Await.result(ready).update(value)
              }
              Future.Unit.delayed(schedule.refreshDelay)(timer).map(_ => start(retry1))
            case r: Return[T] =>
              val value = r()
              ready.setValue(new ReadWriteVar[T](value))
              Future.Unit.delayed(schedule.refreshDelay)(timer).map(_ => start(retry1))
            case _: Throw[_] if retries > 0 =>
              logger.info(
                s"retry ${schedule.loader.name} in ${schedule.retryDelay}. retries:$retries"
              )
              Future.Unit.delayed(schedule.retryDelay)(timer).map(_ => start(retries - 1))
            case _: Throw[_] if ready.isDefined =>
              Future.Unit.delayed(schedule.refreshDelay)(timer).map(_ => start(retry1))
            case t: Throw[_] =>
              logger.info(s"failed to initialize ${schedule.loader.name}")
              ready.setException(t.throwable)
              Future.Unit
          }
      }
    }

    private var task: Future[Unit] = Future.Unit

    lazy val result: Future[ReadWriteVar[T]] = {
      task = start(retry0)
      ready.interruptible()
    }

    override def close(deadline: Time): Future[Unit] = {
      task.raise(new FutureCancelledException)
      task
    }
  }

  def apply[T](schedule: Schedule[T]): Future[Var[T] with Extractable[T]] = {

    val loader = schedule.loader

    val scheduledItem = synchronized {
      if (schedules.contains(loader.name)) {
        val item = schedules.apply(loader.name).asInstanceOf[ScheduledItem[T]]
        item.schedule = schedule
        item
      } else {
        val item = ScheduledItem(schedule)
        schedules = schedules + (loader.name -> item)
        item
      }
    }

    scheduledItem.result
  }

  def names: Seq[String] = schedules.keys.toList

  def dump(): Unit = {
    logger.info(
      schedules.values
        .map(item => s"${item.schedule.loader.name}=${item.ready.isDefined}").mkString(",")
    )
  }

  def ready: Future[Unit] =
    Future
      .join(
        schedules.values.toSeq
          .filter(_.schedule.required)
          .map(_.ready.asInstanceOf[Future[_]])
      )
      .interruptible()

  def stop(name: String): Future[Unit] = synchronized {
    if (schedules.contains(name)) {
      val item = schedules.apply(name)
      schedules = schedules - name
      item.close()
    } else {
      Future.Unit
    }
  }

  override def close(deadline: Time): Future[Unit] = {
    closed.set(true)
    Future.join(schedules.values.map(_.close(deadline)).toSeq)
  }
}
