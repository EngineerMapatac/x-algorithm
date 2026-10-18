package com.twitter.media_understanding.model_proxy.model_descriptors

import com.twitter.conversions.DurationOps._
import com.twitter.cortex_core.{thriftscala => CortexCore}
import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.logging.Logger
import com.twitter.media_understanding.model_proxy.clients.DeepbirdClient
import com.twitter.media_understanding.model_proxy.config.Features
import com.twitter.media_understanding.model_proxy.exception.LabelValidationException
import com.twitter.mediaservices.commons.codec.ThriftByteBufferCodec
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.mediaservices.commons.thriftscala.MediaCategory._
import com.twitter.ml.api.Feature
import com.twitter.ml.api.thriftscala.DataRecord
import com.twitter.util.Try
import java.nio.ByteBuffer
import javax.inject.Inject

class XXNsfwModelDescriptor @Inject() (features: Features) extends ModelDescriptor {
  import Try.OrThrow

  override val model = MediaModel.XxNsfw

  private[this] val log = Logger.get(getClass)
  private[this] val mediaFeatureId = new Feature.Blob("media").getFeatureId
  private[this] val mediaCategoryFeatureId = new Feature.Discrete("meta.media_type").getFeatureId
  private[this] val mediaEncoder = ThriftByteBufferCodec(CortexCore.Media)

  override def supportedCategories: Set[MediaCategory] = Set(
    TweetImage,
    TweetVideo,
    PeriscopeVideo,
    TweetGif,
  )

  override def labels(): Map[String, Long] = {
    Map(
      "nsfw.default.probability" -> new Feature.Continuous("nsfw.default.probability").getFeatureId,
      "nsfw.default.score" -> new Feature.Continuous("nsfw.default.score").getFeatureId,
      "nsfw.default.precision" -> new Feature.Continuous("nsfw.default.precision").getFeatureId,
      "nsfw.default.recall" -> new Feature.Continuous("nsfw.default.recall").getFeatureId,
      "nsfw.default.fpr" -> new Feature.Continuous("nsfw.default.fpr").getFeatureId
    )
  }

  override def validate(dataRecord: DataRecord): Unit = {
    val continuousFeatures = dataRecord.continuousFeatures.get
    labels.foreach {
      case (labelName, labelId) =>
        log.info(s"Validating ${model.name} with label ${labelName}")
        if (!continuousFeatures.contains(labelId)) {
          throw new LabelValidationException(labelName, model.name)
        }
    }
  }

  override def features(
    mediaCategory: Option[MediaCategory],
    media: ByteBuffer
  ): Try[DataRecord] =
    for {
      category <- mediaCategory.orThrow(
        new IllegalArgumentException("The NSFW model requires a media category."))
      modelSupportedMediaCategory <- translateModelSupportedMediaCategory(category)

      blobFeature = Map(
        mediaFeatureId -> mediaEncoder.encode(CortexCore.Media(data = Some(media)))
      )
      discreteFeature = Map(
        mediaCategoryFeatureId -> modelSupportedMediaCategory.getValue.toLong
      )
    } yield {
      DataRecord(blobFeatures = Some(blobFeature), discreteFeatures = Some(discreteFeature))
    }

  override def serviceBuilders() = {
    List(
      DeepbirdClient.Builder(
        "/s/xx-nsfw/xx-nsfw",
        "xx-nsfw",
        8000.milliseconds,
        features.xXNsfwModelV1,
      )
    )
  }
}
