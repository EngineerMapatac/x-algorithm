package com.twitter.botmaker.rule

import java.util.concurrent.TimeUnit
import scala.collection.JavaConverters._
import com.google.common.cache.CacheBuilder
import com.twitter.botmaker.compiler.exceptions.RuntimeFailure
import com.twitter.util.Await
import com.twitter.util.Duration
import com.twitter.util.Future

trait ExprCache {
  def get[T](key: AnyRef, f: => T): T
  def invalidate(key: AnyRef): Unit
  def getIfPresent(key: AnyRef): AnyRef
  def put(key: AnyRef, value: AnyRef): Unit

  def stats: Option[CacheStats] = None
}

trait CacheStats {
  def hitCount: Long
  def missCount: Long
  def loadSuccessCount: Long
  def loadExceptionCount: Long
  def totalLoadTime: Long
  def evictionCount: Long
}

object ExprCache {

  val empty: ExprCache = new ExprCache {
    override def get[T](key: AnyRef, f: => T) = f
    override def invalidate(key: AnyRef) = ()
    override def getIfPresent(key: AnyRef) = null
    override def put(key: AnyRef, value: AnyRef) = ()
  }

  def apply(
    maximumSize: Int = 100000,
    expireAfterAccess: Duration = Duration.Top,
    expireAfterWrite: Duration = Duration.Top,
    concurrencyLevel: Int = 32,
    recordStats: Boolean = false
  ): ExprCache = {

    var builder = CacheBuilder.newBuilder
      .maximumSize(maximumSize)
      .expireAfterAccess(expireAfterAccess.inMillis, TimeUnit.MILLISECONDS)
      .expireAfterWrite(expireAfterWrite.inMillis, TimeUnit.MILLISECONDS)
      .concurrencyLevel(concurrencyLevel)

    if (recordStats) {
      builder = builder.recordStats()
    }

    val cache = builder.build[AnyRef, AnyRef]()

    new ExprCache {

      override def get[T](key: AnyRef, f: => T) = {
        cache.get(key, f.asInstanceOf[AnyRef]) match {
          case t: T => t
          case _ => throw new RuntimeFailure("corrupted data in cache")
        }
      }

      override def invalidate(key: AnyRef) = cache.invalidate(key)
      override def getIfPresent(key: AnyRef): AnyRef = cache.getIfPresent(key)
      override def put(key: AnyRef, value: AnyRef): Unit = cache.put(key, value)

      override def stats: Option[CacheStats] = {
        val stats = cache.stats
        Some(new CacheStats {
          override def hitCount: Long = stats.hitCount

          override def missCount: Long = stats.missCount

          override def loadSuccessCount: Long = stats.loadSuccessCount

          override def loadExceptionCount: Long = stats.loadExceptionCount

          override def totalLoadTime: Long = stats.totalLoadTime

          override def evictionCount: Long = stats.evictionCount

          override def toString: String = stats.toString
        })
      }

      override def toString: String = {
        val sb = new StringBuilder
        cache.asMap.entrySet.asScala foreach { e =>
          e.getValue match {
            case fv: Future[AnyRef] if fv.isDefined =>
              val result = Await.result(fv)
              sb.append(e.getKey).append("=").append(result).append("\n")
            case _: Future[AnyRef] =>
              sb.append(e.getKey).append("=").append("<pending>").append("\n")
            case nv =>
              sb.append(e.getKey).append("=").append(nv).append("\n")
          }
        }
        sb.toString
      }
    }
  }
}
