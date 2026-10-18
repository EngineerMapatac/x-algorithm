package com.twitter.media_understanding.model_proxy.clients

import com.twitter.decider.Feature
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.thrift.ClientId
import com.twitter.media_understanding.common.config.RichFeature
import com.twitter.ml.prediction_service.thriftscala.PredictionRequest
import com.twitter.ml.prediction_service.thriftscala.PredictionResponse
import com.twitter.util.Future

class SwitchingModelClient(
  clientA: ModelClient,
  clientB: ModelClient,
  altClientFeature: Feature,
  sendDarkRequests: Feature = RichFeature.NeverAvailable,
) extends ModelClient {
  val name = s"Switching(${clientA.name},${clientB.name})"

  override def isAvailable =
    if (altClientFeature.isAvailable) clientB.isAvailable else clientA.isAvailable

  override def predictFromModel(request: PredictionRequest): Future[PredictionResponse] =
    if (altClientFeature.isAvailable) {
      if (sendDarkRequests.isAvailable) clientA.predictFromModel(request)
      clientB.predictFromModel(request)
    } else {
      if (sendDarkRequests.isAvailable) clientB.predictFromModel(request)
      clientA.predictFromModel(request)
    }
}

object SwitchingModelClient {
  case class Builder(
    builderA: ModelClient.Builder,
    builderB: ModelClient.Builder,
    altClientFeature: Feature,
    sendDarkRequests: Feature = RichFeature.NeverAvailable,
  ) extends ModelClient.Builder {
    override def build(clientId: ClientId, serviceIdentifier: ServiceIdentifier) =
      new SwitchingModelClient(
        builderA.build(clientId, serviceIdentifier),
        builderB.build(clientId, serviceIdentifier),
        altClientFeature,
        sendDarkRequests,
      )
  }
}
