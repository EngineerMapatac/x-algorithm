package com.twitter.media_understanding.model_proxy.model_descriptors

import com.twitter.conversions.DurationOps._
import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.logging.Logger
import com.twitter.media_understanding.model_proxy.clients.DeepbirdClient
import com.twitter.media_understanding.model_proxy.config.Features
import com.twitter.media_understanding.model_proxy.exception.LabelValidationException
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
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

class FingerprintingV2ModelDescriptor @Inject() (features: Features) extends ModelDescriptor {
  override val model = MediaModel.FingerprintingV2

  private[this] val log = Logger.get(getClass)
  private[this] val mediaFeatureId = new Feature.Tensor(
    "magicpony.media_fingerprinting.image",
    JavaDType.UINT8
  ).getFeatureId

  override def labels(): Map[String, Long] = {
    Map(
      "pixel_variance" -> new Feature.Tensor("pixel_variance", JavaDType.FLOAT).getFeatureId,
      "embedding" -> new Feature.Tensor("embedding", JavaDType.FLOAT).getFeatureId
    )
  }

  override def validate(dataRecord: DataRecord): Unit = {
    val tensors = dataRecord.tensors.get
    labels.foreach {
      case (labelName, labelId) =>
        log.info(s"Validating ${model.name} with label ${labelName}")
        if (!tensors.contains(labelId)) {
          throw new LabelValidationException(labelName, model.name)
        }
    }
  }

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

  override def serviceBuilders() = {
    List(
      DeepbirdClient.Builder(
        "/s/magicpony-fingerprinting/magicpony-fingerprinting",
        "magicpony-fingerprinting",
        3000.milliseconds,
        features.fingerprintingModelV2,
      )
    )
  }
}
