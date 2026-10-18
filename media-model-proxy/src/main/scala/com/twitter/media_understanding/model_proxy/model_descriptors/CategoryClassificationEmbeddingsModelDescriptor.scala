package com.twitter.media_understanding.model_proxy.model_descriptors
import com.google.inject.Inject
import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.media_understanding.model_proxy.clients.DeepbirdClient
import com.twitter.media_understanding.model_proxy.clients.DummyEmbeddingsClient
import com.twitter.media_understanding.model_proxy.clients.SwitchingModelClient
import com.twitter.media_understanding.model_proxy.config.Features
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.ml.api.thriftscala.DataRecord
import com.twitter.ml.api.thriftscala.DataType
import com.twitter.ml.api.thriftscala.GeneralTensor
import com.twitter.ml.api.thriftscala.RawTypedTensor
import com.twitter.ml.api.Feature
import com.twitter.ml.api.{DataType => JavaDType}
import com.twitter.util.Return
import com.twitter.util.Try
import java.nio.ByteBuffer
import com.twitter.conversions.DurationOps._

class CategoryClassificationEmbeddingsModelDescriptor @Inject() (
  features: Features,
  statsReceiver: StatsReceiver)
    extends ModelDescriptor {

  override val model: MediaModel = MediaModel.CategoryClassificationEmbeddings

  private[this] val mediaFeatureId = new Feature.Tensor(
    "media.data",
    JavaDType.UINT8
  ).getFeatureId

  override val unavailableCategories = Seq(
    MediaCategory.TweetGif -> features.gifsForCategoryClassificationEmbeddings,
    MediaCategory.DmGif -> features.gifsForCategoryClassificationEmbeddings,
  )

  override def features(
    mediaCategory: Option[MediaCategory],
    media: ByteBuffer
  ): Try[DataRecord] = {
    val mediaTensor = RawTypedTensor(DataType.Uint8, media)
    val tensors = Map(
      mediaFeatureId -> GeneralTensor.RawTypedTensor(mediaTensor)
    )
    Return(DataRecord(tensors = Some(tensors)))
  }

  override def serviceBuilders() = List(
    SwitchingModelClient.Builder(
      DeepbirdClient.Builder(
        "/s/embeddings-category/embeddings-category-classification",
        "embeddings-category-classification",
        4000.milliseconds,
        features.categoryClassificationEmbeddingsModelV1
      ),
      DummyEmbeddingsClient.Builder(
        "embeddings-category-classification",
        features.categoryClassificationEmbeddingsModelV1,
        dimensions = 256,
        featureId = -1,
        statsReceiver
      ),
      features.dummyCategoryClassificationEmbeddings,
      features.categoryClassificationEmbeddingsAltDarkTraffic,
    )
  )
}
