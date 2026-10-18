package com.twitter.media_understanding.media_model_proxy.loadtest

import com.twitter.cortex_media_annotator.thriftscala.AnnotateRequestResponse
import com.twitter.cortex_media_annotator.thriftscala.AnnotationRequest
import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.dal.client.schema.text.DALTextData
import com.twitter.finagle.thrift.ClientId
import com.twitter.logging.Logger
import com.twitter.scalding_internal.job._
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.scalding._
import com.twitter.scalding.source.TypedText
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.util.Time
import com.twitter.util.{Duration => TwitterDuration}
import com.twitter.conversions.DurationOps._
import twadoop_config.configuration.log_categories.group.cortex.CortexMediaAnnotatorScalaDataset

case class ParsedRow(
  mediaCategory: String,
  mediaId: Long,
  broadcastId: String,
  mediaPath: String,
  originalMediaPath: String,
  models: String)
    extends DALTextData

object MediaModelProxyUniformSamplerJob extends TwitterExecutionApp {
  val DefaultDownsampleRate: Double = 0.01
  val DefaultSampleSeed: Int = 19571103
  val DefaultTTL: Option[TwitterDuration] = None

  private val log = Logger(getClass)

  val clientId: ClientId = ClientId("media-understanding-media-model-proxy-log-sampler-job")

  def annotateToRow(
    annotation: AnnotateRequestResponse
  ): Option[ParsedRow] = {
    try {
      val request: AnnotationRequest = annotation.request
      val mediaId: Long = request.entity.mediaId.get.toLong
      val broadcastId: String = request.entity.broadcastId.getOrElse("")
      val mediaCategory: MediaCategory = request.entity.category.get
      val mediaPath: String = request.entity.blobstorePath.getOrElse("")
      val originalMediaPath: String = request.entity.originalBlobstorePath.getOrElse("")
      val models = request.models.map(_.name).mkString(":")

      Some(
        ParsedRow(mediaCategory.name, mediaId, broadcastId, mediaPath, originalMediaPath, models))
    } catch {
      case e: Exception =>
        log.error(e, s"Sample cannot be converted to row:\n $e")
        None
    }
  }

  def filterWithMediaCategory(
    categories: Set[MediaCategory]
  )(
    annotation: AnnotateRequestResponse
  ): Boolean =
    categories.isEmpty || annotation.request.entity.category.exists(categories.contains)

  def filterWithModels(models: Set[MediaModel])(annotation: AnnotateRequestResponse): Boolean =
    models.isEmpty || annotation.request.models.exists(models.contains)

  override def job: Execution[Unit] = Execution.getArgs.flatMap { args: Args =>
    val downsampleRate: Double = args
      .optional("downsampleRate")
      .map(_.toDouble)
      .getOrElse(DefaultDownsampleRate) match {
      case x if x < 0.0 || x > 1.0 =>
        throw new IllegalArgumentException("downsampleRate should be between 0 and 1")
      case x => x
    }

    val sampleSeed: Int = args
      .optional("sampleSeed")
      .map(_.toInt)
      .getOrElse(DefaultSampleSeed)

    val categories: Set[MediaCategory] =
      args
        .list("media-categories")
        .map(MediaCategory.valueOf)
        .map { el =>
          el.getOrElse(throw new IllegalArgumentException(s"unknown media category $el"))
        }
        .toSet

    val models: Set[MediaModel] =
      args
        .list("models-allowlist")
        .map(MediaModel.valueOf)
        .map { el =>
          el.getOrElse(throw new IllegalArgumentException(s"unknown media model $el"))
        }
        .toSet

    val dateRange = args.list("date") match {
      case Nil =>
        val now = Time.now
        DateRange((now - 14.hours).toDate, (now - 2.hours).toDate)
      case list =>
        DateRange.parse(list)(DateOps.UTC, DateParser.default)
    }

    val outputSink: TypedSink[ParsedRow] = TypedText.csv[ParsedRow](args("output"))

    DAL
      .read(CortexMediaAnnotatorScalaDataset, dateRange).toTypedPipe
      .filter(filterWithMediaCategory(categories))
      .filter(filterWithModels(models))
      .flatMap(annotateToRow)
      .sample(downsampleRate, sampleSeed)
      .shard(1)
      .writeExecution(outputSink)
  }
}
