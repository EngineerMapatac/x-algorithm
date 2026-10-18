package com.twitter.media_understanding.model_proxy.util

import com.twitter.cortex_media_annotator.thriftscala._
import com.twitter.decider.Decider
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.media_understanding.common.helpers.Context
import com.twitter.media_understanding.model_proxy.config.Config
import com.twitter.media_understanding.model_proxy.config.Deciders
import com.twitter.mediaservices.commons.util.MediaCategoryUtil
import com.twitter.servo.util.Scribe
import com.twitter.util.Future

class ScribeLogger(
  category: String,
  config: () => Config,
  decider: Decider,
  statsReceiver: StatsReceiver,
) extends Scriber {
  val reqRepScriber = Scribe(AnnotateRequestResponse, category, statsReceiver)

  def scribe(request: AnnotationRequest, response: AnnotationResponse): Future[Unit] = {
    if (shouldScribeResponse()) {
      reqRepScriber(AnnotateRequestResponse(updateRequest(request), response))
    }

    Future.Unit
  }

  private def updateRequest(request: AnnotationRequest) = {

    val mediaEntity = request.entity.category match {
      case Some(category) if (MediaCategoryUtil.isImage(category)) =>
        request.entity.copy(originalBlobstorePath = request.entity.blobstorePath)

      case _ => request.entity
    }

    request.copy(entity = mediaEntity)
  }

  private def shouldScribeResponse() =
    Context.userId.forall(config().fullLogUserIds.contains) ||
      decider.isAvailable(Deciders.ScribeSampleRate)
}

object NullScriber extends Scriber {
  def scribe(request: AnnotationRequest, response: AnnotationResponse): Future[Unit] = {
    Future.Unit
  }
}
