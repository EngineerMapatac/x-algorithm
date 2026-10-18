package com.twitter.media_understanding.model_proxy.clients

import com.twitter.decider.Feature
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.servo.util.MemoizingStatsReceiver
import com.twitter.finagle.thrift.ClientId
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.ml.api.thriftscala.DataRecord
import com.twitter.ml.api.thriftscala.DataType
import com.twitter.ml.api.thriftscala.GeneralTensor
import com.twitter.ml.api.thriftscala.RawTypedTensor
import com.twitter.ml.prediction_service.thriftscala.PredictionRequest
import com.twitter.ml.prediction_service.thriftscala.PredictionResponse
import com.twitter.util.Future
import java.nio.ByteBuffer
import java.nio.ByteOrder
import scala.util.Random

class DummyEmbeddingsClient(
  override val name: String,
  availabilityFeature: Feature,
  dimensions: Int,
  featureId: Long,
  statsReceiver: StatsReceiver)
    extends ModelClient {

  private[this] val dummyEmbeddingsClientScope = new MemoizingStatsReceiver(
    statsReceiver.scope("dummy-embeddings-client"))
  private[this] val embeddingsCounterStat = dummyEmbeddingsClientScope.counter("total")

  val rng = new Random

  def isAvailable: Boolean = availabilityFeature.isAvailable

  def predictFromModel(request: PredictionRequest): Future[PredictionResponse] = Future {
    val embedding = new Array[Float](dimensions)
    (0 until dimensions).foreach(i => embedding(i) = rng.nextFloat())
    val embeddingBuffer =
      ByteBuffer.allocate(embedding.length * 4).order(ByteOrder.LITTLE_ENDIAN)
    embeddingBuffer.asFloatBuffer.put(embedding).position(0)
    val embeddingDataRecord = RawTypedTensor(
      DataType.Float,
      embeddingBuffer,
      Some(Seq(dimensions)),
    )
    embeddingsCounterStat.incr()

    PredictionResponse(
      DataRecord(
        tensors = Some(
          Map(
            featureId -> GeneralTensor.RawTypedTensor(embeddingDataRecord),
          )),
      )
    )
  }
}

object DummyEmbeddingsClient {
  case class Builder(
    name: String,
    availabilityFeature: Feature,
    dimensions: Int,
    featureId: Int,
    statsReceiver: StatsReceiver)
      extends ModelClient.Builder {
    override def build(clientId: ClientId, serviceIdentifier: ServiceIdentifier) =
      new DummyEmbeddingsClient(
        name,
        availabilityFeature,
        dimensions,
        featureId,
        statsReceiver
      )
  }
}
