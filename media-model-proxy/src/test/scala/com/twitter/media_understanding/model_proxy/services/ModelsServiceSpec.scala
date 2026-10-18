package com.twitter.media_understanding.model_proxy.services

import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.finagle.NoBrokersAvailableException
import com.twitter.finagle.stats.NullStatsReceiver
import com.twitter.media_understanding.model_proxy.clients.ModelClient
import com.twitter.media_understanding.model_proxy.model_descriptors.ModelDescriptor
import com.twitter.media_understanding.model_proxy.model_descriptors.ModelDescriptorRegistry
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.ml.api.thriftscala.DataRecord
import com.twitter.ml.prediction_service.thriftscala.PredictionRequest
import com.twitter.ml.prediction_service.thriftscala.PredictionResponse
import com.twitter.ml.prediction_service.thriftscala.PredictionServiceException
import com.twitter.util.Await
import com.twitter.util.Future
import com.twitter.util.Return
import java.nio.ByteBuffer
import org.mockito.ArgumentMatchers.{any => mockAny}
import org.mockito.Mockito._
import org.scalatest.BeforeAndAfterEach
import org.scalatest.FunSuite
import org.scalatestplus.mockito.MockitoSugar

class ModelsServiceSpec extends FunSuite with MockitoSugar with BeforeAndAfterEach {
  val model = MediaModel.list.head
  val client_1 = mock[ModelClient]
  val client_2 = mock[ModelClient]
  val clientsMap = Map(model -> List(client_1, client_2))
  val descriptors = mock[ModelDescriptorRegistry]
  val deepBirdService = new ModelsService(clientsMap, NullStatsReceiver, descriptors)
  val media = ByteBuffer.wrap("media".getBytes)
  val mediaCategory = MediaCategory.list.head

  override def beforeEach(): Unit = {
    val mockDescriptor = mock[ModelDescriptor]
    when(descriptors.getDescriptor(mockAny[MediaModel])).thenReturn(Some(mockDescriptor))
    when(mockDescriptor.features(Some(mediaCategory), media)).thenReturn(Return(DataRecord()))
    when(mockDescriptor.unavailableCategories).thenReturn(Seq())
    super.beforeEach()
  }

  test("getPrediction correctly returns predictions from multiple cluster of model") {
    when(client_1.isAvailable)
      .thenReturn(true)
    when(client_2.isAvailable)
      .thenReturn(true)
    when(client_1.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.value(PredictionResponse(DataRecord())))
    when(client_2.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.value(PredictionResponse(DataRecord())))

    val result = Await.result(deepBirdService.getPrediction(Some(mediaCategory), model, media))
    assert(result.size == 2)
  }

  test(
    "getPrediction filters out PredictionServiceException if any model in cluster throws PredictionException"
  ) {
    when(client_1.isAvailable)
      .thenReturn(true)
    when(client_2.isAvailable)
      .thenReturn(true)
    when(client_1.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.value(PredictionResponse(DataRecord())))

    when(client_2.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.exception(new PredictionServiceException("")))

    val result = Await.result(deepBirdService.getPrediction(Some(mediaCategory), model, media))
    assert(result.size == 1)
  }

  test(
    "getPrediction filters out PredictionServiceException if any model in cluster throws a non fatal exception"
  ) {
    when(client_1.isAvailable)
      .thenReturn(true)
    when(client_2.isAvailable)
      .thenReturn(true)
    when(client_1.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.value(PredictionResponse(DataRecord())))

    when(client_2.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.exception(new IllegalArgumentException()))

    val result = Await.result(deepBirdService.getPrediction(Some(mediaCategory), model, media))
    assert(result.size == 1)
  }

  test(
    "getPrediction returns empty list of response if all models in cluster throw PredictionException"
  ) {
    when(client_1.isAvailable)
      .thenReturn(true)
    when(client_2.isAvailable)
      .thenReturn(true)
    when(client_1.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.exception(new PredictionServiceException("")))

    when(client_2.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.exception(new PredictionServiceException("")))

    val result = Await.result(deepBirdService.getPrediction(Some(mediaCategory), model, media))
    assert(result.isEmpty)
  }

  test(
    "getPrediction returns empty list of response if all models in cluster throw non fatal exceptions"
  ) {
    when(client_1.isAvailable)
      .thenReturn(true)
    when(client_2.isAvailable)
      .thenReturn(true)
    when(client_1.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.exception(new IllegalArgumentException()))

    when(client_2.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.exception(new IllegalArgumentException()))

    val result = Await.result(deepBirdService.getPrediction(Some(mediaCategory), model, media))
    assert(result.isEmpty)
  }

  test(
    "getPrediction filters out NoBrokersAvailableException if any of the cluster is down or not working"
  ) {
    when(client_1.isAvailable)
      .thenReturn(true)
    when(client_2.isAvailable)
      .thenReturn(true)
    when(client_1.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.exception(new NoBrokersAvailableException()))

    when(client_2.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.value(PredictionResponse(DataRecord())))

    val result = Await.result(deepBirdService.getPrediction(Some(mediaCategory), model, media))
    assert(result.size == 1)
  }

  test(
    "getPrediction correctly remove deepbird client if they are not available and returns prediction response from the filtered") {
    when(client_1.isAvailable)
      .thenReturn(false)
    when(client_2.isAvailable)
      .thenReturn(true)
    when(client_2.predictFromModel(mockAny[PredictionRequest]))
      .thenReturn(Future.value(PredictionResponse(DataRecord())))

    val result1 = Await.result(deepBirdService.getPrediction(Some(mediaCategory), model, media))
    assert(result1.size == 1)

    when(client_2.isAvailable)
      .thenReturn(false)
    when(client_2.isAvailable)
      .thenReturn(false)

    val result2 = Await.result(deepBirdService.getPrediction(Some(mediaCategory), model, media))
    assert(result2.isEmpty)
  }
}
