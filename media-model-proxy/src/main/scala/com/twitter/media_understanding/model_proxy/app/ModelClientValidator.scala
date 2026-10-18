package com.twitter.media_understanding.model_proxy.app

import com.twitter.conversions.DurationOps._
import com.twitter.logging.Logger
import com.twitter.media_understanding.model_proxy.model_descriptors.ModelDescriptorRegistry
import com.twitter.media_understanding.model_proxy.services.ModelsService
import com.twitter.mediaservices.commons.Utils
import com.twitter.mediaservices.commons.thriftscala.MediaCategory.TweetImage
import com.twitter.util.Await
import com.twitter.util.Future
import java.nio.ByteBuffer

class ModelClientValidator(
  registry: ModelDescriptorRegistry,
  deepbirdService: ModelsService) {
  private[this] val log = Logger.get(getClass)
  private[this] val mediaCategory = TweetImage

  def validate(): Unit = {
    val predictions = mediaBytes.flatMap { media =>
      val modelPredictions = registry.getModels().toList.map { model =>
        deepbirdService
          .getPrediction(Some(TweetImage), model, media)
          .map { dataRecord =>
            (model, dataRecord)
          }
      }

      Future.collect(modelPredictions)
    }

    val result = Await.result(predictions, 60.seconds)

    result.foreach {
      case (model, dataRecords) =>
        dataRecords.foreach {
          log.info(s"Validating data record labels for ${model.name} model...")
          registry.getDescriptor(model).get.validate(_)
        }
    }
  }

  private def mediaBytes(): Future[ByteBuffer] = {
    Future.value(Utils.readBytesFromResourceFilePath("sample-image.jpg").get)
  }
}
