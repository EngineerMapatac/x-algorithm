package com.twitter.media_understanding.model_proxy.app

import com.google.inject.Stage
import com.twitter.cortex_media_annotator.thriftscala._
import com.twitter.finatra.mtls.thriftmux.EmbeddedMtlsThriftServer
import com.twitter.inject.server.{FeatureTest => FinatraFeatureTest}
import com.twitter.media_understanding.model_proxy.modules.TestClientModules
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.mediaservices.commons.thriftscala.MediaCategory.TweetImage
import com.twitter.util.Await
import java.nio.ByteBuffer
import org.mockito.Mockito.never
import org.mockito.Mockito.times
import org.mockito.Mockito.verify
import org.scalatest.BeforeAndAfterEach
import org.specs2.mock.Mockito

class FeatureTest extends FinatraFeatureTest with Mockito with BeforeAndAfterEach {

  val server = new EmbeddedMtlsThriftServer(
    twitterServer = new MediaModelProxyThriftServer() {
      override val overrideModules = Seq(TestClientModules)
    },
    stage = Stage.DEVELOPMENT,
    flags = Map(
      "com.twitter.server.wilyns.disable" -> "true",
      "config.overlay" -> "",
      "dtab.add" -> "/$/inet => /$/nil; /zk => /$/nil",
      "decider.base" -> "/test_decider_base.yml"
    )
  )

  lazy val mediaAnnotatorClient =
    server.thriftClient[CortexMediaAnnotator.MethodPerEndpoint](clientId = "testClient")

  import TestClientModules._

  override def beforeEach(): Unit = {
    resetMocks
    super.beforeEach()
  }

  test(
    "getAnnotations should return single prediction when called with blobstore path and single model") {
    val category = TweetImage
    val model = MediaModel.XxNsfw
    val request = AnnotationRequest(
      MediaEntity(
        category = Some(category),
        blobstorePath = Some(mediaPath)
      ),
      Set(model)
    )
    val response = Await.result(mediaAnnotatorClient.getAnnotations(request))

    verify(blobStoreService, times(1)).getMediaByPath(mediaPath)
    verify(deepbirdService, times(1)).getPrediction(Some(category), model, mediaBytes)
    verify(scriber, times(1)).scribe(request, response)

    val annotations = response.annotations.get
    assert(annotations.keySet.size == 1)
    assert(annotations.values.head.size == numClusterPerModel)
  }

  test(
    "getAnnotations should return prediction when called with binary media payload and single model") {
    val category = TweetImage
    val model = MediaModel.XxNsfw
    val request = AnnotationRequest(
      MediaEntity(
        category = Some(category),
        mediaData = Some(mediaBytes)
      ),
      Set(model)
    )
    val response = Await.result(mediaAnnotatorClient.getAnnotations(request))

    verify(blobStoreService, never).getMediaByPath(mediaPath)
    verify(deepbirdService, times(1)).getPrediction(Some(category), model, mediaBytes)
    verify(scriber, times(1)).scribe(request, response)

    val annotations = response.annotations.get
    assert(annotations.keySet.size == 1)
    assert(annotations.values.head.size == numClusterPerModel)
  }

  test(
    "getAnnotations should return should return predictions when called with blobstore path and multiple model"
  ) {
    val request = AnnotationRequest(
      MediaEntity(
        category = Some(TweetImage),
        blobstorePath = Some(mediaPath)
      ),
      models
    )
    val response = Await.result(mediaAnnotatorClient.getAnnotations(request))

    verify(blobStoreService, times(1)).getMediaByPath(mediaPath)
    verify(deepbirdService, times(models.size))
      .getPrediction(any[Option[MediaCategory]], any[MediaModel], any[ByteBuffer])
    verify(scriber, times(1)).scribe(request, response)

    val annotations = response.annotations.get
    assert(annotations.keySet.size == models.size)
    for (model <- annotations.keySet) {
      assert(annotations(model).size == numClusterPerModel)
    }
  }

  test(
    "getAnnotations should return should return predictions when called with binary media payload and multiple model"
  ) {
    val request = AnnotationRequest(
      MediaEntity(
        category = Some(TweetImage),
        mediaData = Some(mediaBytes)
      ),
      models
    )
    val response = Await.result(mediaAnnotatorClient.getAnnotations(request))

    verify(blobStoreService, never).getMediaByPath(mediaPath)
    verify(deepbirdService, times(models.size))
      .getPrediction(any[Option[MediaCategory]], any[MediaModel], any[ByteBuffer])
    verify(scriber, times(1)).scribe(request, response)

    val annotations = response.annotations.get
    assert(annotations.keySet.size == models.size)
    for (model <- annotations.keySet) {
      assert(annotations(model).size == numClusterPerModel)
    }
  }

  test(
    "getAnnotations should return a prediction when called without media id or category for a supported model") {
    val model = MediaModel.Fingerprinting
    val request = AnnotationRequest(
      MediaEntity(
        blobstorePath = Some(mediaPath)
      ),
      Set(model)
    )
    val response = Await.result(mediaAnnotatorClient.getAnnotations(request))

    verify(deepbirdService, times(1)).getPrediction(None, model, mediaBytes)

    val annotations = response.annotations.get
    assert(annotations.keySet.size == 1)
    assert(annotations.values.head.size == numClusterPerModel)
  }
}
