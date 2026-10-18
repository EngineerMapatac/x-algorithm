package com.twitter.media_understanding.model_proxy.model_descriptors

import com.twitter.media_understanding.model_proxy.config.NullFeatures
import com.twitter.finagle.stats.NullStatsReceiver
import com.twitter.mediaservices.commons.thriftscala.MediaCategory.TweetVideo
import com.twitter.ml.api.Feature
import com.twitter.ml.api.{DataType => JavaDType}
import java.nio.ByteBuffer
import org.scalatest.FunSuite
import org.scalatestplus.mockito.MockitoSugar

class CategoryClassificationEmbeddingsModelDescriptorSpec extends FunSuite with MockitoSugar {
  private[this] val mediaFeatureId = new Feature.Tensor(
    "media.data",
    JavaDType.UINT8
  ).getFeatureId

  private[this] val descriptor =
    new CategoryClassificationEmbeddingsModelDescriptor(new NullFeatures, new NullStatsReceiver)

  test("Features correctly returns dataRecord with Model supported Media Categories") {
    val byteBuff = ByteBuffer.wrap("media_bytes".getBytes)
    val dataRecord = descriptor.features(Some(TweetVideo), byteBuff).get()
    assert(dataRecord.tensors.get.contains(mediaFeatureId))
  }
}
