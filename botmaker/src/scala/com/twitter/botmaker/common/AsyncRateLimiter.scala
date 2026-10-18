package com.twitter.botmaker.common

import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.atomic._
import com.twitter.common.util.Clock
import com.twitter.util.Duration
import com.twitter.util.Future
import com.twitter.util.Promise
import com.twitter.util.Timer
import scala.util.control.NoStackTrace

trait AsyncRateLimiter {

  def acquire(): Future[Unit]

}

case class AsyncRateLimiterMaxWaitersExceededException(msg: String)
    extends Throwable(msg)
    with NoStackTrace

object AsyncRateLimiter {

  val noop: AsyncRateLimiter = new AsyncRateLimiter {
    override def acquire(): Future[Unit] = Future.Unit
  }

  def apply(timer: Timer, duration: Duration)(rate: => Int): AsyncRateLimiter =
    apply(timer, Clock.SYSTEM_CLOCK, duration, Int.MaxValue)(rate)

  def apply(
    timer: Timer,
    clock: Clock,
    duration: Duration,
    maxWaiters: Int
  )(
    rate: => Int
  ): AsyncRateLimiter = {

    val flushInterval = duration / 8 + Duration.fromMilliseconds(1)
    apply(timer, clock, duration, flushInterval, maxWaiters)(rate)
  }

  def apply(
    timer: Timer,
    clock: Clock,
    duration: Duration,
    flushInterval: Duration,
    maxWaiters: Int
  )(
    rate: => Int
  ): AsyncRateLimiter = new AsyncRateLimiter {

    @volatile private[this] var startTime: Long = 0
    private[this] val counter = new AtomicInteger(0)
    private[this] val queued = new AtomicInteger(0)

    private[this] val queue = new ConcurrentLinkedQueue[Promise[Unit]]
    private[this] val flushing = new AtomicBoolean(false)
    private[this] val flushTask = timer.schedule(flushInterval)(flush)

    private[this] val cachedException = Future.exception(
      AsyncRateLimiterMaxWaitersExceededException(s"Max waiters exceeds $maxWaiters")
    )

    def acquire(): Future[Unit] = {

      val r = rate
      if (r <= 0) {
        cachedException
      } else if (counter.incrementAndGet() <= r) {
        Future.Unit
      } else if (queued.get >= maxWaiters) {
        cachedException
      } else {
        val promise = new Promise[Unit]
        queued.incrementAndGet()
        queue.offer(promise)
        promise
      }
    }

    private def flush(): Unit = {
      if (flushing.compareAndSet(false, true)) {
        try {
          val currentTime = clock.nowMillis
          if (currentTime - startTime >= duration.inMilliseconds) {
            var newCounter = 0
            var continue = true
            while (continue && newCounter < rate) {
              val top = queue.poll()
              continue = if (top != null) {
                queued.decrementAndGet()
                if (top.setDone) {
                  newCounter += 1
                }
                true
              } else {
                false
              }
            }
            startTime = currentTime
            counter.set(newCounter)
          } else {
            var continue = true
            while (continue) {
              val top = queue.peek()
              continue = if (top == null) {
                false
              } else if (top.isDefined) {
                queued.decrementAndGet()
                queue.poll()
                true
              } else if (counter.incrementAndGet() < rate) {
                queued.decrementAndGet()
                queue.poll()
                top.setDone()
                true
              } else {
                false
              }
            }
          }
        } finally {
          flushing.set(false)
        }
      }
    }
  }

}
