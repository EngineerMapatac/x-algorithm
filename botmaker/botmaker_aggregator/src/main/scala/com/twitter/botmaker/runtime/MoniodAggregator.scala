package com.twitter.botmaker.runtime

import com.google.common.cache._
import com.twitter.util._
import java.util.concurrent.TimeUnit
import scala.collection.JavaConverters._
import com.twitter.finagle.stats.Counter

abstract class MonoidAggregator[I, Stat, O](
  runtime: TwitterRuntime,
  val flushInterval: Duration,
  val flushSlices: Int,
  val cacheSize: Long,
  val cacheTTL: Duration)
    extends Closable {

  type StatKey = AnyRef

  type Entry = java.util.Map.Entry[StatKey, Stat]

  private val statsReceiver = runtime.statsReceiver

  private val clock = runtime.clock

  private val timer = runtime.timer

  protected val enqueueCounter: Counter = statsReceiver.counter("enqueue")
  protected val flushCounter: Counter = statsReceiver.counter("flush")
  protected val successCounter: Counter = statsReceiver.counter("success")
  protected val failureCounter: Counter = statsReceiver.counter("failure")
  protected val latencyStat: com.twitter.finagle.stats.Stat = statsReceiver.stat("latency")
  protected val underflowCounter: Counter = statsReceiver.counter("underflow")

  private val sizeGauge = statsReceiver.addGauge("queue_size") {
    cache.size
  }

  private val sizeRemoval = statsReceiver.counter("cache_size_removal")
  private val expiredRemoval = statsReceiver.counter("cache_expired_removal")
  private val otherRemoval = statsReceiver.counter("cache_other_removal")

  protected lazy val cache: LoadingCache[StatKey, Stat] = CacheBuilder.newBuilder
    .maximumSize(cacheSize)
    .expireAfterAccess(cacheTTL.inMilliseconds, TimeUnit.MILLISECONDS)
    .removalListener(new RemovalListener[StatKey, Stat] {
      override def onRemoval(removalNotification: RemovalNotification[StatKey, Stat]): Unit = {
        removalNotification.getCause match {
          case RemovalCause.SIZE => sizeRemoval.incr()
          case RemovalCause.EXPIRED => expiredRemoval.incr()
          case _ => otherRemoval.incr()
        }
      }
    })
    .asInstanceOf[CacheBuilder[StatKey, Stat]]
    .build(new CacheLoader[StatKey, Stat] {
      override def load(k: StatKey): Stat = zero
    })

  def post(key: StatKey, amount: Any): Future[Unit] = {
    add(key, amount.asInstanceOf[I])
  }

  def add(key: StatKey, amount: I): Future[Unit] = {
    enqueueCounter.incr
    val stat = cache.get(key)
    plus(stat, amount)
    Future.Unit
  }

  protected def zero: Stat

  def plus(stat: Stat, amount: I): Unit

  def flush(batch: Stream[Entry]): Future[Unit]

  private val flushSliceInterval = flushInterval / Math.max(1, flushSlices)

  protected def nowIndex: Long = clock.nowMillis / flushInterval.inMilliseconds

  private val closing = new Promise[Unit]()

  @volatile
  private var completedIndex = nowIndex

  private def run(): Future[Unit] = {

    val triggers = Seq(
      Future.sleep(flushSliceInterval)(timer),
      closing
    )

    Future.select(triggers) flatMap { _ =>
      val next = completedIndex + 1
      val nowMillis = clock.nowMillis
      val now = nowIndex
      val isClosing = closing.isDefined
      if (isClosing || now >= next) {
        if (now > next) {
          underflowCounter.incr()
        }
        Future.Unit flatMap { _ =>
          flushCounter.incr
          val stream = cache.asMap().entrySet.asScala.toStream
          flush(stream)
        } onSuccess { _ =>
          successCounter.incr
        } onFailure { t =>
          failureCounter.incr
        } rescue {
          case _ => Future.Unit
        } ensure {
          completedIndex = now
          latencyStat.add(clock.nowMillis - nowMillis)
        } flatMap {
          case _ if isClosing => Future.Unit
          case _ => run
        }
      } else {
        run
      }
    }
  }

  private var flusher: Future[Unit] = run()

  lazy val ready: Future[Unit] = flusher.interruptible

  override def close(deadline: com.twitter.util.Time): Future[Unit] = {
    closing.updateIfEmpty(Return.Unit)
    flusher
  }
}
