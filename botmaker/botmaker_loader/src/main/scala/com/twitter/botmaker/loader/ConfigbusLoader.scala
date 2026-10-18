package com.twitter.botmaker.loader

import com.twitter.configbus.client.ConfigSnapshot
import com.twitter.configbus.client.ConfigSource
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.io.Buf
import com.twitter.util._
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicReference
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import scala.util.control.NonFatal

class ConfigbusLoader[T](
  loaderName: String,
  configSource: ConfigSource,
  configFileName: String,
  statsReceiver: StatsReceiver,
  deserialize: Buf => T,
  defaultIfNotFound: Option[T] = None,
  newValueSetter: (Option[T], T) => T = (_: Option[T], newValue: T) => newValue,
  loadedInformationForLogging: T => String = (_: T) => "")
    extends AbstractLoader[T](loaderName, statsReceiver)
    with Closable {

  private[this] val configValRef = new AtomicReference[Option[T]](None)
  private[this] val closableRef = new AtomicReference[Closable](Closable.nop)
  private[this] val readyPromise = Promise[Unit]()

  private[this] val logger: Logger = LoggerFactory.getLogger(getClass)

  private[this] val scoped = statsReceiver.scope(loaderName.toLowerCase)

  private[this] val updateFailureCounter = scoped.counter("update_failure")
  private[this] val updateSuccessCounter = scoped.counter("update_success")
  private[this] val updateFallbackSuccessCounter = scoped.counter("update_fallback_success")

  private[this] val latestUpdateValid = new AtomicBoolean(false)
  private[this] val latestUpdateValidGauge = scoped.addGauge("latest_update_is_valid") {
    if (latestUpdateValid.get()) 1 else 0
  }
  private[this] val loadFailureCounter = scoped.counter("load_config_failure")
  private[this] val loadSuccessCounter = scoped.counter("load_config_success")

  initialize()

  override def close(deadline: Time): Future[Unit] = {
    closableRef.get.close(deadline)
  }

  override def fetch(): Future[T] = {
    Future.const(getValue)
  }

  def getValue: Try[T] = Try {
    (configValRef.get, defaultIfNotFound) match {
      case (Some(currentValue), _) =>
        logger.debug("{}_ConfigbusLoader: fetched", loaderName)
        currentValue
      case (_, Some(defaultValue)) =>
        logger.debug(
          "{}_ConfigbusLoader failed to initialize successfully; fetching with " +
            "default value",
          loaderName)
        defaultValue
      case _ =>
        throw new RuntimeException(
          s"${name}_ConfigbusLoader is not ready, failing to fetch: $configFileName")
    }
  }

  private def initialize(): Unit = {
    logger.info(
      s"${name}_ConfigbusLoader: Initializing the configuration subscriber for $configFileName"
    )

    val activity = configSource.subscribe(configFileName)

    val updatingClosable = activity.values.map {
      case Return(ConfigSnapshot(_, buf)) =>
        logger.info(s"$loaderName ConfigbusLoader Detected Config Snapshot from $configFileName")
        parseConfigFileForNewValue(buf)
      case Throw(ex) => Throw(ex)
    } respond {
      case Return(newValue) =>
        setNewValueForLoader(newValue)

        updateSuccessCounter.incr()
        latestUpdateValid.set(true)
        readyPromise.setDone()
        logger.info(s"$loaderName finished setting the new value from $configFileName")
      case Throw(e) if defaultIfNotFound.isDefined && configValRef.get.isEmpty =>
        logger.info(
          s"$loaderName ConfigbusLoader encountered issue parsing new value from " +
            s"$configFileName, will use default value",
          e)
        updateFallbackSuccessCounter.incr()
        latestUpdateValid.set(false)
        readyPromise.setDone()
      case Throw(e) =>
        updateFailureCounter.incr()
        latestUpdateValid.set(false)
        readyPromise.updateIfEmpty(Throw(e))
        logger.error(
          s"$loaderName ConfigbusLoader Detected Config Change -> Got an error " +
            s"from $configFileName.",
          e)
    }
    closableRef.set(updatingClosable)
  }

  override def ready: Future[Unit] = readyPromise.interruptible()

  private def parseConfigFileForNewValue(buf: Buf): Try[T] = Try {
    val previousReference = configValRef.get
    val rawDeserialized = deserialize(buf)
    newValueSetter(previousReference, rawDeserialized)
  } onFailure { _ =>
    loadFailureCounter.incr()
  }

  private def setNewValueForLoader(value: T): Unit = {
    configValRef.set(Some(value))

    loadSuccessCounter.incr()
    logger.info(
      "%s_ConfigbusLoader loaded %s".format(
        loaderName,
        loadedInformationForLogging(value)
      )
    )
  }

}
