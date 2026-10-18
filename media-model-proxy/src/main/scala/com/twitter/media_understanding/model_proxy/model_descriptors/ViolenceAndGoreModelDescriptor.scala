package com.twitter.media_understanding.model_proxy.model_descriptors

import com.twitter.conversions.DurationOps._
import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.logging.Logger
import com.twitter.media_understanding.model_proxy.clients.DeepbirdClient
import com.twitter.media_understanding.model_proxy.config.Features
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.mediaservices.commons.thriftscala.MediaCategory.AvatarImage
import com.twitter.mediaservices.commons.thriftscala.MediaCategory.BannerImage
import com.twitter.mediaservices.commons.thriftscala.MediaCategory.TweetGif
import com.twitter.mediaservices.commons.thriftscala.MediaCategory.TweetImage
import com.twitter.mediaservices.commons.thriftscala.MediaCategory.TweetVideo
import com.twitter.ml.api.thriftscala.DataRecord
import com.twitter.ml.api.thriftscala.DataType
import com.twitter.ml.api.thriftscala.GeneralTensor
import com.twitter.ml.api.thriftscala.RawTypedTensor
import com.twitter.ml.api.Feature
import com.twitter.ml.api.{DataType => JavaDType}
import com.twitter.util.Return
import com.twitter.util.Try
import java.nio.ByteBuffer
import javax.inject.Inject

class ViolenceAndGoreModelDescriptor @Inject() (features: Features) extends ModelDescriptor {
  override val model = MediaModel.ViolenceAndGore

  private[this] val log = Logger.get(getClass)
  private[this] val mediaCategoryFeatureId = new Feature.Discrete("meta.media_type").getFeatureId
  private[this] val mediaFeatureId = new Feature.Tensor(
    "violence_and_gore.image",
    JavaDType.UINT8
  ).getFeatureId

  override def supportedCategories: Set[MediaCategory] = Set(
    AvatarImage,
    BannerImage,
    TweetImage,
    TweetVideo,
    TweetGif
  )

  override def features(
    mediaCategory: Option[MediaCategory],
    media: ByteBuffer
  ): Try[DataRecord] = {
    val mediaTensor = RawTypedTensor(DataType.Uint8, media)
    val tensors = Map(
      mediaFeatureId -> GeneralTensor.RawTypedTensor(mediaTensor)
    )

    val discreteFeature = for {
      category <- mediaCategory
      modelSupportedMediaCategory <- translateModelSupportedMediaCategory(category).toOption

      discreteFeature = Map(
        mediaCategoryFeatureId -> modelSupportedMediaCategory.getValue.toLong
      )
    } yield discreteFeature

    Return(DataRecord(tensors = Some(tensors), discreteFeatures = discreteFeature))
  }

  override def serviceBuilders() = {
    List(
      DeepbirdClient.Builder(
        "/s/violence-and-gore/violence-and-gore",
        "violence-and-gore",
        3000.milliseconds,
        features.violenceAndGoreModelV1,
      )
    )
  }
}
