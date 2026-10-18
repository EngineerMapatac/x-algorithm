package com.twitter.media_understanding.model_proxy.integrationtest

import com.twitter.app.GlobalFlag
import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.media_understanding.common.AwaitResult
import com.twitter.media_understanding.common.integrationtest.Injection
import com.twitter.media_understanding.common.integrationtest.IntegrationTestSuite
import com.twitter.media_understanding.common.integrationtest.MediaServices
import com.twitter.media_understanding.common.integrationtest.PollUntil
import com.twitter.media_understanding.common.integrationtest.TestImages
import com.twitter.media_understanding.common.integrationtest.TestUsers
import com.twitter.mediaservices.commons.photurkey.thriftscala.PrivacyType
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.ml.api.thriftscala.DataRecord
import org.scalatest.matchers.should.Matchers
import scala.collection.Map
import scala.io.Source.fromFile

class MediaModelProxyTest
    extends IntegrationTestSuite
    with Matchers
    with Injection
    with MediaServices
    with MediaModelProxyService
    with TestImages
    with TestUsers
    with PollUntil
    with AwaitResult {
  import MediaModelProxyTest._

  val modelsPath: String =
    "media-understanding/media-model-proxy/integrationtest/src/test/resources/com/twitter/media_understanding/model_proxy/integrationtest/models.txt"
  val expectedModels: Seq[String] = fromFile(modelsPath).getLines.toSeq

  val allModels: String = expectedModels.mkString(":")

  test("Media Model Proxy get annotations") {
    if (Disable())
      cancel("Media Model Proxy integration tests disabled")

    val media = loadMedia(imagePaths.head)

    infoHeader("Fetching test user")

    val userId = createTestUser().await

    val tweetId = 0L

    infoHeader("Uploading images")

    val (mediaSource, metadata) = uploadAndProcessMedia(
      media,
      userId,
      tweetId,
      category = MediaCategory.TweetImage,
      privacyType = PrivacyType.Public
    ).await

    val mediaId = mediaSource.getCompatibilityMediaKey.mediaId.toString
    val mediaPathPrefix = MediaPathPrefix()

    val response = getAnnotations(
      mediaCategory = Some(mediaSource.getCompatibilityMediaKey.category),
      mediaPath = Some(s"/$mediaPathPrefix/media-ingestor/${mediaId}/sanitized"),
      mediaId = Some(mediaId),
      originalPath = None,
      broadcastId = None,
      models = allModels
    )

    response.annotations.map { models: Map[MediaModel, Seq[DataRecord]] =>
      val modelNames = models.keys.toSeq.map(_.name)

      modelNames.sorted shouldEqual expectedModels.sorted
      models.foreach {
        case (model, prediction) =>
          withClue(s"prediction for model ${model.name},") {
            prediction shouldNot be(empty)
          }
      }

      info("Integration test successful.")
    }
  }
}

object MediaModelProxyTest {
  object Disable
      extends GlobalFlag[Boolean](
        default = true,
        "Disable the Media Model Proxy integration tests"
      ) { override val name = "media-model-proxy.disable" }

  object MediaPathPrefix
      extends GlobalFlag[String](
        default = "",
        "The media path prefix used for the selected environment."
      ) { override val name = "media-path-prefix" }
}
