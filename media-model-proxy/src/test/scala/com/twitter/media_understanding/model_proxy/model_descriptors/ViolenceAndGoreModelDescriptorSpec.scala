package com.twitter.media_understanding.model_proxy.model_descriptors

import com.twitter.media_understanding.model_proxy.config.NullFeatures
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.mediaservices.commons.thriftscala.MediaCategory._
import com.twitter.ml.api.Feature
import com.twitter.ml.api.{DataType => JavaDType}
import java.nio.ByteBuffer
import org.scalatest.FunSuite
import org.scalatest.Matchers
import org.scalatestplus.mockito.MockitoSugar

class ViolenceAndGoreModelDescriptorSpec extends FunSuite with MockitoSugar with Matchers {
  private[this] val mediaCategoryFeatureId = new Feature.Discrete("meta.media_type").getFeatureId
  private[this] val mediaFeatureId = new Feature.Tensor(
    "violence_and_gore.image",
    JavaDType.UINT8
  ).getFeatureId
  private[this] val descriptor = new ViolenceAndGoreModelDescriptor(new NullFeatures)

  test("Features correctly returns DataRecord with model supported media categories") {
    val byteBuff = ByteBuffer.wrap("media_bytes".getBytes)
    val modelSupportedCategories: Set[MediaCategory] = Set(
      AvatarImage,
      BannerImage,
      TweetImage,
      TweetVideo,
      TweetGif
    )

    modelSupportedCategories.foreach(mediaCategory => {
      val byteBuff = ByteBuffer.wrap("media_bytes".getBytes)

      val dataRecord = descriptor.features(Some(mediaCategory), byteBuff).get()

      assert(dataRecord.tensors.get.contains(mediaFeatureId))
      assert(
        dataRecord.discreteFeatures.get(mediaCategoryFeatureId) == mediaCategory.getValue.toLong)
      assert(dataRecord.discreteFeatures.size == 1)
    })
  }

  test("All unsupported GIF should map to TweetGif") {
    val byteBuff = ByteBuffer.wrap("media_bytes".getBytes)

    val dataRecord = descriptor.features(Some(DmGif), byteBuff).get()

    assert(dataRecord.discreteFeatures.get(mediaCategoryFeatureId) == TweetGif.getValue.toLong)

  }

  test("All unsupported Videos should map to TweetVideo") {
    val byteBuff = ByteBuffer.wrap("media_bytes".getBytes)

    val dataRecord = descriptor.features(Some(AmplifyVideo), byteBuff).get()

    assert(dataRecord.discreteFeatures.get(mediaCategoryFeatureId) == TweetVideo.getValue.toLong)
  }

  test("All unsupported Images should map to TweetImage") {
    val byteBuff = ByteBuffer.wrap("media_bytes".getBytes)

    val dataRecord = descriptor.features(Some(DmGroupImage), byteBuff).get()

    assert(dataRecord.discreteFeatures.get(mediaCategoryFeatureId) == TweetImage.getValue.toLong)
  }

  test(
    "Features should not throw an exception when unsupported media category(not an image/gif/video) is passed") {
    val byteBuff = ByteBuffer.wrap("media_bytes".getBytes)

    val dataRecord = descriptor.features(Some(TwitterBroadcast), byteBuff).get()

    assert(dataRecord.tensors.get.contains(mediaFeatureId))
    assert(dataRecord.discreteFeatures.isEmpty)
  }

  test("Features should not throw an exception when no media category is passed") {
    val byteBuff = ByteBuffer.wrap("media_bytes".getBytes)

    val dataRecord = descriptor.features(None, byteBuff).get()

    assert(dataRecord.tensors.get.contains(mediaFeatureId))
    assert(dataRecord.discreteFeatures.isEmpty)
  }
}
