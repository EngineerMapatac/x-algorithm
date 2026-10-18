package com.twitter.botmaker.common

import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicLong
import java.util.concurrent.TimeUnit
import java.util.logging.Level
import java.util.logging.Logger
import java.io.PrintWriter
import java.io.StringWriter
import com.google.common.cache._
import com.twitter.botmaker.compiler.exceptions.FunctionFailure
import com.twitter.botmaker.rule.getStackTrace
import com.twitter.common.util.Clock
import com.twitter.util._

case class RatelimitedLogger(
  logger: Logger,
  intervalMilis: Int = 60000,
  maximumCacheItem: Int = 10000,
  expireAfterAccess: Duration = Duration.fromSeconds(600),
  expireAfterWrite: Duration = Duration.fromSeconds(600),
  concurrencyLevel: Int = 8,
  clock: Clock = Clock.SYSTEM_CLOCK) {

  case class State(latch: AtomicBoolean, skipped: AtomicLong, var timestamp: Long)

  private[this] val cache = CacheBuilder.newBuilder
    .maximumSize(maximumCacheItem)
    .expireAfterAccess(expireAfterAccess.inMillis, TimeUnit.MILLISECONDS)
    .expireAfterWrite(expireAfterWrite.inMillis, TimeUnit.MILLISECONDS)
    .concurrencyLevel(concurrencyLevel)
    .build(new CacheLoader[AnyRef, State] {
      override def load(k: AnyRef): State = State(new AtomicBoolean(false), new AtomicLong(0L), 0L)
    })

  def acquire(key: AnyRef): Boolean = {
    val state = cache.get(key)
    if (state.latch.compareAndSet(false, true)) {
      try {
        val b = clock.nowMillis / intervalMilis
        if (state.timestamp != b) {
          val skipped = state.skipped.getAndSet(0L)
          if (skipped > 0) {
            val last = state.timestamp * intervalMilis
            logger.info(s"$skipped logs had been skipped since $last for $key.")
          }
          state.timestamp = b
          true
        } else {
          state.skipped.incrementAndGet
          false
        }
      } finally {
        state.latch.set(false)
      }
    } else {
      state.skipped.incrementAndGet
      false
    }
  }

  def setLevel(newLevel: Level): Unit = logger.setLevel(newLevel)

  def debug(msg: String): Unit = logger.fine(msg)

  def info(msg: String): Unit = logger.info(msg)

  def warning(msg: String): Unit = logger.warning(msg)

  def warning(msg: String, t: Throwable): Unit = {
    logger.warning(throwableText(msg, t))
  }

  private def throwableText(msg: String, t: Throwable): String = t match {
    case ff: FunctionFailure =>
      val sw = new StringWriter
      val pw = new PrintWriter(sw, true)
      pw.println(msg)
      pw.println(ff.getStackFrameMessage)
      ff.getCause.printStackTrace(pw)
      sw.getBuffer.toString
    case _ =>
      s"$msg, ${getStackTrace(t)}"
  }

  def debug(key: AnyRef, msg: String): Unit = if (logger.isLoggable(Level.FINE) && acquire(key)) {
    logger.fine(msg)
  }

  def debug(key: AnyRef, msg: String, t: Throwable): Unit =
    if (logger.isLoggable(Level.FINE) && acquire(key)) {
      logger.fine(throwableText(msg, t))
    }

  def info(key: AnyRef, msg: String): Unit = if (acquire(key)) {
    info(msg)
  }

  def warning(key: AnyRef, msg: String): Unit = if (acquire(key)) {
    warning(msg)
  }

  def warning(key: AnyRef, msg: String, t: Throwable): Unit = if (acquire(key)) {
    warning(msg, t)
  }

}

object RatelimitedLogger {

  def getLogger(name: String): RatelimitedLogger = {
    val logger = Logger.getLogger(name)
    RatelimitedLogger(logger)
  }
}
