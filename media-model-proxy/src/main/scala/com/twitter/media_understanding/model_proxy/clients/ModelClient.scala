package com.twitter.media_understanding.model_proxy.clients

import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.thrift.ClientId
import com.twitter.ml.prediction_service.thriftscala.PredictionRequest
import com.twitter.ml.prediction_service.thriftscala.PredictionResponse
import com.twitter.util.Future

trait ModelClient {
  def isAvailable: Boolean
  def predictFromModel(request: PredictionRequest): Future[PredictionResponse]
  def name: String
}

object ModelClient {
  trait Builder {
    def build(clientId: ClientId, serviceIdentifier: ServiceIdentifier): ModelClient
  }
}
