package com.twitter.botmaker.runtime

import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicLong
import java.util.regex.Pattern
import com.google.common.cache.CacheBuilder
import com.google.common.cache.CacheLoader
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.RegexTimeoutException
import com.twitter.botmaker.compiler.exceptions.RuntimeFailure
import com.twitter.botmaker.compiler.tuples.Pair
import com.twitter.botmaker.runtime.config.RegexOptionConfigs
import com.twitter.botmaker.runtime.thriftjava.RegexConfig
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util._
import java.{util => ju}

case class RegexThrottledFailure(msg: String) extends RuntimeFailure(msg)

case class RegexRegistry(
  name: String,
  config: RegexConfig,
  runtime: TwitterRuntime,
  scoped: StatsReceiver)
    extends ServiceRegistry[RegexConfig]
    with RegexCache {

  override def close(deadline: Time): Future[Unit] = Future.Unit

  private[this] val requestsCounter = scoped.counter("requests")
  private[this] val timeoutRequestsCounter = scoped.counter("requests", "timeout")
  private[this] val errorRequestsCounter = scoped.counter("requests", "error")
  private[this] val skippedRequestsCounter = scoped.counter("requests", "skipped")
  private[this] val requestsLatencyStat = scoped.stat("requests", "latency")
  private[this] val cacheMissesCounter = scoped.counter("cache", "miss")

  private[this] val pendingRequests = new AtomicLong(0L)
  private[this] val pendingRequestsGauge = scoped.addGauge("requests", "pending") {
    pendingRequests.get.toFloat
  }

  private[this] val timeoutDuration = Duration.fromMilliseconds(config.timeoutMilis)

  private[this] val patterns = CacheBuilder.newBuilder
    .maximumSize(config.maximumCacheItems)
    .expireAfterAccess(config.cacheExpirationMillis, TimeUnit.MILLISECONDS)
    .build(new CacheLoader[Pair[String, Integer], Try[Pattern]]() {
      override def load(key: Pair[String, Integer]): Try[Pattern] = {
        cacheMissesCounter.incr()
        Try {
          Pattern.compile(key.getFirst, key.getSecond)
        }
      }
    })

  private[this] val regexCache = RegexCache.of(config.timeoutMilis.toInt, patterns)

  private def apply[T](call: => T): T = {

    requestsCounter.incr()

    if (pendingRequests.get >= config.maximumPendingRequests) {
      skippedRequestsCounter.incr()
      throw RegexThrottledFailure("maximum pending requests reached")
    }

    val startTime = runtime.clock.nowMillis

    try {
      pendingRequests.incrementAndGet
      call
    } catch {
      case t: RegexTimeoutException =>
        runtime.logger.warning("RegexTimeout", t.toString)
        timeoutRequestsCounter.incr()
        throw t
      case t: Throwable =>
        runtime.logger.warning("RegexError", t.toString)
        errorRequestsCounter.incr()
        throw t
    } finally {
      pendingRequests.decrementAndGet
      requestsLatencyStat.add(runtime.clock.nowMillis - startTime)
    }
  }

  override def matches(context: Context[_], regex: String, str: String): Boolean = apply {
    regexCache.matches(context, regex, str)
  }

  override def extract(context: Context[_], regex: String, str: String): ju.List[String] = apply {
    regexCache.extract(context, regex, str)
  }

  override def split(context: Context[_], regex: String, str: String): ju.List[String] = apply {
    regexCache.split(context, regex, str)
  }

  override def replace(
    context: Context[_],
    str: String,
    target: String,
    replacement: String
  ): String =
    apply {
      regexCache.replace(context, str, target, replacement)
    }

}

trait RegexContainer extends ServiceContainer with RegexCache {
  self =>

  private val registryKey = "RegexOptions"

  override type _CONFIGS_TYPE <: RegexOptionConfigs

  def isReusable(config: RegexConfig): Boolean = {
    isReusable[RegexConfig, RegexRegistry](registryKey, config)
  }

  def regexCache: RegexCache =
    serviceRegistry[RegexRegistry](registryKey).getOrElse(RegexCache.DEFAULT)

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val regexOptons = if (isReusable(configs.getRegexOptions)) {
      logger.info("reuse Regex registry")
      serviceRegistry[RegexRegistry](registryKey).get
    } else {
      logger.info(s"create Regex registry")
      RegexRegistry(registryKey, configs.getRegexOptions, this, scoped.scope("Regex"))
    }
    Seq(regexOptons) ++ super.build(configs, scoped)
  }

  override def matches(context: Context[_], regex: String, str: String): Boolean =
    regexCache.matches(context, regex, str)

  override def extract(context: Context[_], regex: String, str: String): ju.List[String] =
    regexCache.extract(context, regex, str)

  override def split(context: Context[_], regex: String, str: String): ju.List[String] =
    regexCache.split(context, regex, str)

  override def replace(
    context: Context[_],
    str: String,
    target: String,
    replacement: String
  ): String =
    regexCache.replace(context, str, target, replacement)
}
