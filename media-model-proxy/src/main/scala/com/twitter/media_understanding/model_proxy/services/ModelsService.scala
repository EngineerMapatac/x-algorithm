package com.twitter.media_understanding.model_proxy.services

import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.finagle.NoBrokersAvailableException
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.logging.Logger
import com.twitter.media_understanding.model_proxy.clients.ModelClient
import com.twitter.media_understanding.model_proxy.exception._
import com.twitter.media_understanding.model_proxy.model_descriptors.ModelDescriptor
import com.twitter.media_understanding.model_proxy.model_descriptors.ModelDescriptorRegistry
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.ml.api.thriftscala.DataRecord
import com.twitter.ml.prediction_service.thriftscala._
import com.twitter.servo.util.MemoizingStatsReceiver
import com.twitter.util.Future
import com.twitter.util.Return
import com.twitter.util.Throw
import com.twitter.util.Try
import java.nio.ByteBuffer
import scala.util.control.NonFatal

class ModelsService(
  clientMapping: Map[MediaModel, List[ModelClient]],
  statsReceiver: StatsReceiver,
  registry: ModelDescriptorRegistry,
) {
  import Try.OrThrow

  private[this] val log = Logger.get()
  private[this] val stats = new MemoizingStatsReceiver(
    statsReceiver.scope("annotate_getPrediction")
  )

  val greyScaleErrorString =
    "!(reader->getNativeColorModel() == imagecore::kColorModel_RGBA || " +
      "reader->getNativeColorModel() == imagecore::kColorModel_RGBX)"

  def getPrediction(
    category: Option[MediaCategory],
    model: MediaModel,
    media: ByteBuffer
  ): Future[Seq[DataRecord]] = {
    val requestAndClient = Future.const(
      for {
        descriptor <-
          registry
            .getDescriptor(model)
            .orThrow(new IllegalArgumentException(s"Unknown model ${model.name} requested"))
        dataRecord <- descriptor.features(category, media)
        filteredClient =
          if (isModelAvailable(descriptor, category)) clientMapping(model).filter { _.isAvailable }
          else Seq()
      } yield (PredictionRequest(dataRecord), filteredClient)
    )

    requestAndClient.flatMap {
      case (predictionRequest, filteredClient) =>
        val result = filteredClient.map { client =>
          val name = client.name
          client
            .predictFromModel(predictionRequest).rescue {
              case ex: PredictionServiceException =>
                val msg =
                  s"Error in getting prediction response from deepbird model ${model.name}\n$ex"
                if (msg.contains(greyScaleErrorString)) {
                  Future.exception(new GreyScaleImageException(msg))
                } else {
                  Future.exception(ex)
                }
            }.onFailure {
              case _: NoBrokersAvailableException =>
                stats.scope(name).counter("no_broker_avail_failure").incr()
              case _: PredictionServiceException =>
                stats.scope(name).counter("prediction_service_exception").incr()
                stats
                  .scope(name).counter("failure").incr()
              case _ => stats.scope(name).counter("failure").incr()
            }.onSuccess { _ =>
              stats.scope(name).counter("success").incr()
            }.ensure {
              stats.scope(name).counter("total").incr()
            }
        }

        Future
          .collectToTry(result)
          .map { responses: Seq[Try[PredictionResponse]] =>
            responses
              .filter { response => filterException(response, model) }
              .map { predictionResponseTry => predictionResponseTry.get().prediction }
          }
    }
  }

  private def isModelAvailable(
    descriptor: ModelDescriptor,
    category: Option[MediaCategory]
  ): Boolean =
    category.toSeq
      .flatMap { cat => descriptor.unavailableCategories.filter(_._1 == cat) }
      .map(_._2)
      .forall { _.isAvailable }

  private def filterException(prediction: Try[PredictionResponse], model: MediaModel) = {
    prediction match {
      case Return(_) => true
      case Throw(ex) =>
        ex match {
          case _: NoBrokersAvailableException =>
            log.error(ex, s"NoBrokersAvailableException in ${model.name}")
            false

          case NonFatal(_) =>
            log.error(ex, s"Failure in model ${model.name}")
            false

          case _ => true
        }
    }
  }
}
