package com.twitter.botmaker.common

import java.util.concurrent.atomic._
import java.util.concurrent.ConcurrentLinkedQueue

import scala.annotation.tailrec
import scala.collection.mutable.ArrayBuffer

import com.twitter.common.util.Clock
import com.twitter.finagle.stats._
import com.twitter.util.Future

case class BatchQueueFullException(msg: String) extends Throwable(msg)

trait BatchDispatcher[T] {

  def queueSize: Int = 1024
  def batchSize: Int = 16
  def concurrentSize: Int = 1

  def rateLimiter: AsyncRateLimiter = AsyncRateLimiter.noop

  def statsReceiver: StatsReceiver
  def clock: Clock = Clock.SYSTEM_CLOCK

  val batchSizeStat: Stat = statsReceiver.stat("batch_size")
  val enqueueCounter: Counter = statsReceiver.counter("enqueues")
  val flushCounter: Counter = statsReceiver.counter("flushes")
  val successCounter: Counter = statsReceiver.counter("successes")
  val failureCounter: Counter = statsReceiver.counter("failures")
  val latencyStat: Stat = statsReceiver.stat("latency")
  val errorCounter: Counter = statsReceiver.counter("errors")

  val queueSizeStat: Gauge = statsReceiver.addGauge("queue_size") {
    this.queueSize
  }

  val pendingSizeStat: Gauge = statsReceiver.addGauge("pending_size") {
    this.running.get
  }

  private[this] val running = new AtomicInteger(0)
  private[this] val queued = new AtomicInteger(0)
  private[this] val flushing = new AtomicBoolean(false)
  private[this] val queue = new ConcurrentLinkedQueue[Seq[T]]

  @tailrec
  final def flush(): Unit = {

    var batches: List[Seq[T]] = Nil
    var locked = false

    if (!queue.isEmpty && flushing.compareAndSet(false, true)) {
      try {

        locked = true

        while (!queue.isEmpty && running.get() < concurrentSize) {

          val batch = new ArrayBuffer[T](batchSize)
          var continue = true

          while (continue) {
            val top = queue.peek
            continue = if (top != null && batch.size == 0) {
              batch ++= queue.poll
              batch.size < batchSize
            } else if (top != null && batch.size + top.size <= batchSize) {
              batch ++= queue.poll
              batch.size < batchSize
            } else {
              false
            }
          }

          if (batch.size > 0) {
            running.incrementAndGet()
            queued.addAndGet(-1 * batch.size)
            batches = batch :: batches
          }
        }
      } finally {
        flushing.set(false)
      }
    }

    if (batches.size > 0) {
      batches foreach { batch =>
        Stat
          .timeFuture(latencyStat)(
            rateLimiter
              .acquire()
              .flatMap(_ => {
                flushCounter.incr()
                batchSizeStat.add(batch.size)
                dispatch(batch)
              })
          )
          .onSuccess(_ => {
            successCounter.incr()
          })
          .onFailure(_ => {
            failureCounter.incr()
          })
          .ensure {
            running.decrementAndGet()
            flushIt()
          }
      }

    } else if (locked && running.get() == 0) {
      flush()
    }
  }

  def add(requests: Seq[T]): Unit = {

    val length = queued.get
    if (length >= queueSize) {
      errorCounter.incr()
      throw BatchQueueFullException(s"BatchQueue size exceeds $length")
    } else {
      enqueueCounter.incr()
      queued.addAndGet(requests.size)
      queue.offer(requests)
      flush()
    }
  }

  def dispatch(batch: Seq[T]): Future[Unit]

  private[this] def flushIt() = flush()
}
