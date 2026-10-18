package com.twitter.media_understanding.media_model_proxy.loadtest

import com.twitter.conversions.DurationOps._
import com.twitter.finagle.thrift.RichClientParam
import com.twitter.finagle.util.DefaultTimer
import com.twitter.iago.legacy.Type.ParrotService
import com.twitter.iago.legacy._
import com.twitter.iago.legacy.Type.ParrotServerFlags
import com.twitter.iago.legacy.ThriftRecordProcessor
import com.twitter.iago.internal.legacy.ThriftParrotConfig
import com.twitter.logging.Logger
import org.apache.thrift.protocol.TBinaryProtocol
import com.twitter.cortex_media_annotator.thriftscala.AnnotationRequest
import com.twitter.cortex_media_annotator.thriftscala.CortexMediaAnnotator
import com.twitter.cortex_media_annotator.thriftscala.MediaEntity
import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.iago.util.Distribution
import com.twitter.iago.util.SlowStartPoissonProcess
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.app.GlobalFlag
import com.twitter.util.Duration

object SlowStart
    extends GlobalFlag[Duration](1.minute, "Slow start duration for SlowStartPoissonProcess")

class MediaModelProxyLoadTestConfig(config: ParrotServerFlags) extends ThriftParrotConfig(config) {
  def distribution(rate: Int): Distribution = new SlowStartPoissonProcess(rate, SlowStart())

  val recordProcessor = new MediaModelProxyLoadTest(service)
}

class MediaModelProxyLoadTest(
  parrotService: ParrotService[ParrotRequest, Array[Byte]])
    extends ThriftRecordProcessor(parrotService) {

  val log = Logger.get(getClass)

  implicit val timer = DefaultTimer

  val client = new CortexMediaAnnotator.FinagledClient(
    service,
    RichClientParam(
      new TBinaryProtocol.Factory()
    )
  )

  override def start(): Unit = {
    log.info("Load test started.")
  }

  object OptStr {
    def unapply(value: String): Option[Option[String]] =
      if (value.isEmpty) Some(None) else Some(Some(value))
  }

  override def processLine(line: String): Unit = {
    line.split(",").toList match {
      case mediaCategory ::
          OptStr(mediaId) ::
          OptStr(broadcastId) ::
          OptStr(mediaPath) ::
          OptStr(originalPath) ::
          models ::
          Nil =>
        val request: AnnotationRequest = AnnotationRequest(
          MediaEntity(
            MediaCategory.valueOf(mediaCategory),
            mediaPath,
            mediaId,
            None,
            originalPath,
            broadcastId
          ),
          models.split(":").flatMap(MediaModel.valueOf).toSet
        )

        val response = client.getAnnotations(request)

        response.within(1.minute).onFailure { throwable =>
          log.info(s"Failure:\n  Request: $line\n  Response: $throwable")
        }
      case _ =>
        log.error(s"Log line >>>$line<<< could not be parsed.")
    }
  }

  override def shutdown(): Unit = {
    log.info("Load test completed.")
  }
}
