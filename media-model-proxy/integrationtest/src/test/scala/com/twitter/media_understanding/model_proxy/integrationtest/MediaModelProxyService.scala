package com.twitter.media_understanding.model_proxy.integrationtest

import com.twitter.inject.Logging
import com.twitter.media_understanding.common.integrationtest.Injection
import com.twitter.media_understanding.common.integrationtest.MediaServices
import com.twitter.media_understanding.common.modules.MediaModelProxyServiceModule
import com.twitter.cortex_media_annotator.{thriftscala => mmpThrift}
import com.twitter.conversions.DurationOps._
import com.twitter.cortex_media_annotator.thriftscala.AnnotationRequest
import com.twitter.cortex_media_annotator.thriftscala.AnnotationResponse
import com.twitter.cortex_media_annotator.thriftscala.MediaEntity
import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.util.Await

trait MediaModelProxyService extends Logging { self: Injection with MediaServices =>
  addModules(
    MediaModelProxyServiceModule
  )

  lazy val mmp = injector.instance[mmpThrift.CortexMediaAnnotator.MethodPerEndpoint]

  def getAnnotations(
    mediaCategory: Option[MediaCategory],
    mediaPath: Option[String],
    mediaId: Option[String],
    originalPath: Option[String],
    broadcastId: Option[String],
    models: String
  ): AnnotationResponse = {

    val request = AnnotationRequest(
      MediaEntity(
        mediaCategory,
        mediaPath,
        mediaId,
        None,
        originalPath,
        broadcastId
      ),
      models.split(":").flatMap(MediaModel.valueOf).toSet
    )

    val response = mmp.getAnnotations(request)

    Await
      .result(response, 60.seconds)
  }
}
