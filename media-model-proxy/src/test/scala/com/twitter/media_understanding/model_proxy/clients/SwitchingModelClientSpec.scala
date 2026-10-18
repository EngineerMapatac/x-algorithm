package com.twitter.media_understanding.model_proxy.clients

import com.twitter.decider.Feature
import com.twitter.media_understanding.common.AwaitResult
import com.twitter.media_understanding.common.DoReturn
import com.twitter.media_understanding.common.config.RichFeature
import com.twitter.ml.prediction_service.thriftscala.PredictionRequest
import com.twitter.ml.prediction_service.thriftscala.PredictionResponse
import com.twitter.util.Future
import org.mockito.ArgumentMatchers.{eq => _eq, _}
import org.mockito.Mockito._
import org.scalatest.matchers.should.Matchers
import org.scalatest.wordspec.AnyWordSpec
import org.scalatestplus.mockito.MockitoSugar

class SwitchingModelClientSpec
    extends AnyWordSpec
    with MockitoSugar
    with AwaitResult
    with DoReturn
    with Matchers {
  class Fixture {
    val clientA = mock[ModelClient]
    val clientB = mock[ModelClient]
    def altClientFeature: Feature = RichFeature.NeverAvailable
    def darkTrafficFeature: Feature = RichFeature.NeverAvailable

    lazy val client = new SwitchingModelClient(
      clientA = clientA,
      clientB = clientB,
      altClientFeature = altClientFeature,
      sendDarkRequests = darkTrafficFeature,
    )
  }

  "SwitchingModelClient" should {
    "call the first service" in {
      val f = new Fixture

      val req = mock[PredictionRequest]
      val resp = mock[PredictionResponse]

      when(f.clientA.predictFromModel(any())) thenReturn Future.value(resp)

      f.client.predictFromModel(req).await shouldBe resp

      verify(f.clientA).predictFromModel(_eq(req))
      verify(f.clientB, never).predictFromModel(any())
    }

    "call the alternate service" in {
      val f = new Fixture {
        override def altClientFeature = RichFeature.AlwaysAvailable
      }

      val req = mock[PredictionRequest]
      val resp = mock[PredictionResponse]

      when(f.clientB.predictFromModel(any())) thenReturn Future.value(resp)

      f.client.predictFromModel(req).await shouldBe resp

      verify(f.clientB).predictFromModel(_eq(req))
      verify(f.clientA, never).predictFromModel(any())
    }

    "send dark traffic to the alternate service" in {
      val f = new Fixture {
        override def darkTrafficFeature = RichFeature.AlwaysAvailable
      }

      val req = mock[PredictionRequest]
      val respA = mock[PredictionResponse]
      val respB = mock[PredictionResponse]

      when(f.clientA.predictFromModel(any())) thenReturn Future.value(respA)
      when(f.clientB.predictFromModel(any())) thenReturn Future.value(respB)

      f.client.predictFromModel(req).await shouldBe respA

      verify(f.clientA).predictFromModel(_eq(req))
      verify(f.clientB).predictFromModel(_eq(req))

    }

    "send dark traffic to the main service when alternate is selected" in {
      val f = new Fixture {
        override def altClientFeature = RichFeature.AlwaysAvailable
        override def darkTrafficFeature = RichFeature.AlwaysAvailable
      }

      val req = mock[PredictionRequest]
      val respA = mock[PredictionResponse]
      val respB = mock[PredictionResponse]

      when(f.clientA.predictFromModel(any())) thenReturn Future.value(respA)
      when(f.clientB.predictFromModel(any())) thenReturn Future.value(respB)

      f.client.predictFromModel(req).await shouldBe respB

      verify(f.clientB).predictFromModel(_eq(req))
      verify(f.clientA).predictFromModel(_eq(req))

    }
  }
}
