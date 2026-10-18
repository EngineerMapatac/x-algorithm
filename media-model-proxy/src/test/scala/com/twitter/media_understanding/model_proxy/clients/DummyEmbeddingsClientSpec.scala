package com.twitter.media_understanding.model_proxy.clients

import com.twitter.decider.Feature
import com.twitter.media_understanding.common.AwaitResult
import com.twitter.media_understanding.common.DoReturn
import com.twitter.media_understanding.common.config.RichFeature
import com.twitter.ml.api.thriftscala.DataType
import com.twitter.ml.api.thriftscala.GeneralTensor
import com.twitter.ml.prediction_service.thriftscala.PredictionRequest
import com.twitter.finagle.stats.InMemoryStatsReceiver
import org.scalatest.OptionValues
import org.scalatest.matchers.should.Matchers
import org.scalatest.wordspec.AnyWordSpec
import org.scalatestplus.mockito.MockitoSugar

class DummyEmbeddingsClientSpec
    extends AnyWordSpec
    with MockitoSugar
    with AwaitResult
    with DoReturn
    with OptionValues
    with Matchers {
  class Fixture {
    def availabilityFeature: Feature = RichFeature.AlwaysAvailable
    def featureId = -1L
    def dimensions = 256
    val stats = new InMemoryStatsReceiver()

    lazy val client =
      new DummyEmbeddingsClient("test-config", availabilityFeature, dimensions, featureId, stats)
  }

  "DummyEmbeddingsClient" should {
    "return a random embedding" in {
      val f = new Fixture

      val req = mock[PredictionRequest]

      val resp = f.client.predictFromModel(req).await

      resp.prediction.tensors.value.get(f.featureId).value match {
        case GeneralTensor.RawTypedTensor(tensor) =>
          tensor.shape.value shouldBe Seq(f.dimensions)
          tensor.dataType shouldBe DataType.Float
        case _ =>
          fail("Not a raw typed tensor")
      }

      f.stats.counters.get(Seq("dummy-embeddings-client", "total")) shouldBe Some(1)
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
