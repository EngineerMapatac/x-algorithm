package com.twitter.media_understanding.model_proxy.clients

import com.twitter.cortex.deepbird.thriftscala.DeepbirdPredictionService
import com.twitter.decider.Feature
import com.twitter.media_understanding.common.config.RichFeature
import com.twitter.media_understanding.common.AwaitResult
import com.twitter.media_understanding.common.DoReturn
import com.twitter.ml.prediction_service.thriftscala.PredictionRequest
import com.twitter.ml.prediction_service.thriftscala.PredictionResponse
import com.twitter.util.Future
import org.mockito.ArgumentMatchers.{eq => _eq, _}
import org.mockito.Mockito._
import org.scalatest.matchers.should.Matchers
import org.scalatest.wordspec.AnyWordSpec
import org.scalatestplus.mockito.MockitoSugar

class DeepbirdClientSpec
    extends AnyWordSpec
    with MockitoSugar
    with AwaitResult
    with DoReturn
    with Matchers {
  class Fixture {
    val service = mock[DeepbirdPredictionService.MethodPerEndpoint]
    def availabilityFeature: Feature = RichFeature.AlwaysAvailable

    lazy val client =
      new DeepbirdClient(service, "test-config", availabilityFeature)
  }

  "PredictionServiceClient" should {
    "call the underlying service" in {
      val f = new Fixture

      val req = mock[PredictionRequest]
      val resp = mock[PredictionResponse]

      when(f.service.predictFromModel(any(), any())) thenReturn Future.value(resp)

      f.client.predictFromModel(req).await shouldBe resp

      verify(f.service).predictFromModel(_eq(req), any())
    }

    "return available if the feature is enabled" in {
      val f = new Fixture

      f.client.isAvailable shouldBe true
    }

    "return unavailable if the feature is disabled" in {
      val f = new Fixture {
        override def availabilityFeature = RichFeature.NeverAvailable
      }

      f.client.isAvailable shouldBe false
    }
  }
}
