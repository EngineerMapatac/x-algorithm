package com.twitter.media_understanding.model_proxy.model_descriptors

import com.twitter.media_understanding.model_proxy.config.NullFeatures
import com.twitter.mediaservices.commons.thriftscala.MediaCategory._
import com.twitter.ml.api.Feature
import com.twitter.ml.api.{DataType => JavaDType}
import java.nio.ByteBuffer
import org.scalatest.FunSuite
import org.scalatestplus.mockito.MockitoSugar

class CategoryClassificationModelDescriptorSpec extends FunSuite with MockitoSugar {
  private[this] val mediaFeatureId = new Feature.Tensor(
    "magicpony.category_classification.image",
    JavaDType.UINT8
  ).getFeatureId
  private[this] val descriptor = new CategoryClassificationModelDescriptor(new NullFeatures)

  test("features correctly returns dataRecord with Model supported Media Categories") {
    val byteBuff = ByteBuffer.wrap("media_bytes".getBytes)
    val dataRecord = descriptor.features(Some(TweetVideo), byteBuff).get()
    assert(dataRecord.tensors.get.get(mediaFeatureId).isDefined)
  }
}
