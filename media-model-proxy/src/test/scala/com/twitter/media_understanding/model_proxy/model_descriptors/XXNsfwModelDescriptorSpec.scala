package com.twitter.media_understanding.model_proxy.model_descriptors

import com.twitter.cortex_media_annotator.thriftscala.BadRequest
import com.twitter.cortex_media_annotator.thriftscala.BadRequestCode
import com.twitter.media_understanding.model_proxy.config.NullFeatures
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.mediaservices.commons.thriftscala.MediaCategory._
import com.twitter.ml.api.Feature
import com.twitter.util.Throw
import java.nio.ByteBuffer
import org.scalatest.FunSuite
import org.scalatest.Matchers
import org.scalatestplus.mockito.MockitoSugar

class XXNsfwModelDescriptorSpec extends FunSuite with MockitoSugar with Matchers {
  private[this] val mediaCategoryFeatureId = new Feature.Discrete("meta.media_type").getFeatureId
  private[this] val descriptor = new XXNsfwModelDescriptor(new NullFeatures)

  test("features correctly returns dataRecord with Model supported Media Categories") {
    val byteBuff = ByteBuffer.wrap("test".getBytes)
    val modelSupportedCategories: Set[MediaCategory] = Set(
      TweetImage,
      TweetVideo,
      PeriscopeVideo,
      TweetGif
    )

    modelSupportedCategories.foreach(mediaCategory => {
      val dataRecord = descriptor.features(Some(mediaCategory), byteBuff).get()
      assert(
        dataRecord.discreteFeatures.get(mediaCategoryFeatureId) == mediaCategory.getValue.toLong)
      assert(dataRecord.discreteFeatures.size == 1)
      assert(dataRecord.blobFeatures.size == 1)
    })

    val dataRecordWithUnsupportedGifCategory = descriptor.features(Some(DmGif), byteBuff).get()
    assert(
      dataRecordWithUnsupportedGifCategory.discreteFeatures
        .get(mediaCategoryFeatureId) == TweetGif.getValue.toLong)

    val dataRecordWithUnsupportedVideoCategory =
      descriptor.features(Some(DmVideo), byteBuff).get()
    assert(
      dataRecordWithUnsupportedVideoCategory.discreteFeatures
        .get(mediaCategoryFeatureId) == TweetVideo.getValue.toLong)

    val dataRecordWithUnsupportedImageCategory =
      descriptor.features(Some(DmGroupImage), byteBuff).get()
    assert(
      dataRecordWithUnsupportedImageCategory.discreteFeatures
        .get(mediaCategoryFeatureId) == TweetImage.getValue.toLong)
  }

  test(
    "Features should not throw an exception when unsupported media category(not an image/gif/video) is passed") {
    val byteBuff = ByteBuffer.wrap("test".getBytes)

    val dataRecord = descriptor.features(Some(TwitterBroadcast), byteBuff)

    dataRecord shouldBe Throw(
      BadRequest(
        "XxNsfw model does not support media category TwitterBroadcast",
        Some(BadRequestCode.InvalidInput)
      )
    )
  }

  test("features throws an exception when no media category is passed") {
    val byteBuff = ByteBuffer.wrap("test".getBytes)

    an[IllegalArgumentException] should be thrownBy descriptor.features(None, byteBuff).get()
  }
}
