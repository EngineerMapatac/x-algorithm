package com.twitter.media_understanding.model_proxy.exception

import com.twitter.cortex_media_annotator.thriftscala.BadRequest
import com.twitter.cortex_media_annotator.thriftscala.BadRequestCode._
import com.twitter.finagle.Failure
import com.twitter.finagle.FailureFlags
import com.twitter.mediaservices.commons._
import com.twitter.videobird.thriftscala.{BadRequest => VideoBirdBadRequest}

object Failures {
  val LoadSheddingFailure = Failure(
    "Request discarded to shed load.",
    FailureFlags.Rejected | FailureFlags.NonRetryable
  )
}

class MediaNotFoundException(msg: String) extends BadRequest(msg, Some(MediaNotFound))

class VideoBirdExtractImageNotFoundException(msg: String)
    extends BadRequest(msg, Some(MediaNotFound))

class MediaCategoryNotSupportedException(msg: String) extends BadRequest(msg, Some(InvalidInput))

class GreyScaleImageException(msg: String) extends BadRequest(msg, Some(MediaNotSupported))

class LabelValidationException(label: String, model: String)
    extends IllegalArgumentException(s"Label $label missing in model $model")

object RuntimeExceptionTransform extends ExceptionTransformer {
  override def transform = {
    case e: FileTooLargeException =>
      MisuseExceptionInfo(BadRequest(e.toString, Some(FileTooLarge)))

    case e: VideoBirdBadRequest =>
      MisuseExceptionInfo(BadRequest(e.toString, Some(InvalidInput)))

    case e: MediaNotFoundException =>
      MisuseExceptionInfo(BadRequest(e.toString, Some(MediaNotFound)))

    case e: BadRequest =>
      MisuseExceptionInfo(e)
  }

  override def getStatName: PartialFunction[Exception, String] = {
    case e: BadRequest => exceptionName(e, e.code.map(_.name).getOrElse("none"))
  }
}
