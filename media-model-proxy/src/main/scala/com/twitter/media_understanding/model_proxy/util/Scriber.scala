package com.twitter.media_understanding.model_proxy.util

import com.twitter.cortex_media_annotator.thriftscala.AnnotationRequest
import com.twitter.cortex_media_annotator.thriftscala.AnnotationResponse
import com.twitter.util.Future

trait Scriber {
  def scribe(request: AnnotationRequest, response: AnnotationResponse): Future[Unit]
}
