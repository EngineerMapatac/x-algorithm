package com.twitter.media_understanding.model_proxy.controllers

import com.twitter.cortex_media_annotator.thriftscala.CortexMediaAnnotator._
import com.twitter.cortex_media_annotator.thriftscala._
import com.twitter.finagle.Service
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.finatra.thrift.Controller
import com.twitter.logging.Logger
import com.twitter.media_understanding.model_proxy.exception.RuntimeExceptionTransform
import com.twitter.media_understanding.model_proxy.services.BlobStoreService
import com.twitter.media_understanding.model_proxy.services.ModelsService
import com.twitter.media_understanding.model_proxy.util.Scriber
import com.twitter.mediaservices.commons.ThriftServer
import com.twitter.servo.util.MemoizingStatsReceiver
import com.twitter.util.Future
import javax.inject.Inject
import org.slf4j.MDC

class ProxyController @Inject() (
  blobStoreService: BlobStoreService,
  deepbirdService: ModelsService,
  scriber: Scriber,
  statsReceiver: StatsReceiver)
    extends Controller(CortexMediaAnnotator) {

  private[this] val thriftServer = new ThriftServer(statsReceiver, Some(RuntimeExceptionTransform))
  private[this] val log = Logger.get(getClass)
  private[this] val mediaCategoryScope = new MemoizingStatsReceiver(
    statsReceiver.scope("media_category")
  )
  private[this] val modelCountStat = statsReceiver.stat("model_count")

  val getAnnotations: Service[GetAnnotations.Args, GetAnnotations.SuccessType] = {
    handle(GetAnnotations) { args: GetAnnotations.Args =>
      thriftServer.track("getAnnotation") {
        val request = args.request
        val mediaPath = request.entity.blobstorePath
        val models = request.models
        val mediaCategory = request.entity.category
        val mediaId = request.entity.mediaId
        val mediaData = request.entity.mediaData

        val optionOfMediaCategoryName = mediaCategory.map { _.name }
        val mediaCategoryName = optionOfMediaCategoryName.getOrElse("no-category")

        optionOfMediaCategoryName.foreach { x => MDC.put("mediaCategory", x.toLowerCase) }
        mediaId.foreach(MDC.put("mediaId", _))

        log.debug(s"Getting predictions for request $request")

        modelCountStat.add(models.size)

        val mediaBytes = mediaData match {
          case Some(data) => Future.value(data)
          case None =>
            blobStoreService.getMediaByPath(
              mediaPath.getOrElse(
                throw new IllegalArgumentException(
                  "Either media_data or media_path must be provided.")
              )
            )
        }

        mediaBytes
          .flatMap { media =>
            val modelPredictions = models.map { model =>
              deepbirdService
                .getPrediction(mediaCategory, model, media)
                .map { dataRecords =>
                  (model, dataRecords)
                }
            }.toList

            Future
              .collect(modelPredictions)
              .map { predictions =>
                AnnotationResponse(datarecord = Map(), annotations = Some(predictions.toMap))
              }
          }
          .onFailure { ex =>
            mediaCategoryScope.scope(mediaCategoryName).counter("failure").incr()
            log.error(ex, s"Error getting predictions for request $request")
          }
          .onSuccess { response =>
            mediaCategoryScope.scope(mediaCategoryName).counter("success").incr()
            scriber.scribe(request, response)
          }
          .ensure {
            mediaCategoryScope.scope(mediaCategoryName).counter("total").incr()
          }
      }
    }
  }
}
