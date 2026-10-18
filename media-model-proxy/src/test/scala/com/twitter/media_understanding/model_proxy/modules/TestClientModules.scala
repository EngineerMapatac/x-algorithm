package com.twitter.media_understanding.model_proxy.modules

import com.google.inject.Provides
import com.google.inject.Singleton
import com.twitter.cortex_media_annotator.thriftscala.AnnotationRequest
import com.twitter.cortex_media_annotator.thriftscala.AnnotationResponse
import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.inject.DeprecatedMockito
import com.twitter.inject.TwitterModule
import com.twitter.media_understanding.model_proxy.services.BlobStoreService
import com.twitter.media_understanding.model_proxy.services.ModelsService
import com.twitter.media_understanding.model_proxy.util.Scriber
import com.twitter.mediaservices.commons.config.Env
import com.twitter.mediaservices.commons.thriftscala.MediaCategory._
import com.twitter.ml.api.thriftscala.DataRecord
import com.twitter.util.Future
import java.nio.ByteBuffer
import org.mockito.Mockito.when

object TestClientModules extends TwitterModule with DeprecatedMockito {
  val blobStoreService = mock[BlobStoreService]
  val deepbirdService = mock[ModelsService]
  val scriber = mock[Scriber]

  val dataRecord = DataRecord()
  val mediaBytes = ByteBuffer.wrap("media".getBytes())
  val mediaPath = "mediaPath"
  val mediaCategories =
    Set(TweetImage, TweetVideo, PeriscopeVideo, TweetGif).map(Some(_)) ++ Set(None)

  val models: Set[MediaModel] = MediaModel.list.toSet
  val numClusterPerModel = 3

  @Provides
  @Singleton
  def providesBlobStoreService(): BlobStoreService = blobStoreService

  @Provides
  @Singleton
  def providesEnv(): Env = Env.Test

  @Provides
  @Singleton
  def providesDeepbirdService(): ModelsService = deepbirdService

  @Provides
  @Singleton
  def providesScriber(): Scriber = scriber

  private def stubMocks(): Unit = {
    when(blobStoreService.getMediaByPath(mediaPath)).thenReturn(Future.value(mediaBytes))

    when(scriber.scribe(any[AnnotationRequest], any[AnnotationResponse])).thenReturn(Future.Unit)

    mediaCategories.foreach(category => {
      models.foreach(model => {
        when(deepbirdService.getPrediction(category, model, mediaBytes))
          .thenReturn(Future.value(List.fill(numClusterPerModel)(dataRecord)))
      })
    })
  }

  def resetMocks(): Unit = {
    Seq(
      blobStoreService,
      deepbirdService,
      scriber
    ).foreach(reset(_))

    stubMocks()
  }
}
