package com.twitter.media_understanding.model_proxy.model_descriptors

import com.twitter.conversions.DurationOps._
import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.logging.Logger
import com.twitter.media_understanding.model_proxy.clients.DeepbirdClient
import com.twitter.media_understanding.model_proxy.config.Features
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

class HatefulSymbolRecognitionModelDescriptor @Inject() (features: Features)
    extends ModelDescriptor {
  override val model = MediaModel.HatefulSymbolRecognition

  private[this] val log = Logger.get(getClass)
  private[this] val mediaFeatureId = new Feature.Tensor(
    "magicpony.hateful_symbol_recognition.image",
    JavaDType.UINT8
  ).getFeatureId

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
        "/s/magicpony-hateful/magicpony-hateful-symbol-recognition",
        "magicpony-hateful-symbol-recognition",
        3000.milliseconds,
        features.hatefulSymbolRecognitionModelV1,
      )
    )
  }
}
