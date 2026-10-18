package com.twitter.media_understanding.model_proxy.clients

import com.twitter.cortex.deepbird.thriftscala.DeepbirdPredictionService
import com.twitter.decider.Feature
import com.twitter.finagle.ThriftMux
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.mtls.client.MtlsStackClient._
import com.twitter.finagle.service.ResponseClassifier
import com.twitter.finagle.thrift.ClientId
import com.twitter.media_understanding.model_proxy.config.Tunables
import com.twitter.ml.prediction_service.thriftscala.PredictionRequest
import com.twitter.ml.prediction_service.thriftscala.PredictionResponse
import com.twitter.util.Duration
import com.twitter.util.Future

class DeepbirdClient(
  client: DeepbirdPredictionService.MethodPerEndpoint,
  override val name: String,
  availabilityFeature: Feature,
) extends ModelClient {

  def isAvailable: Boolean = availabilityFeature.isAvailable

  def predictFromModel(request: PredictionRequest): Future[PredictionResponse] = {
    client.predictFromModel(request)
  }
}

object DeepbirdClient {

  case class Builder(
    destination: String,
    name: String,
    timeout: Duration,
    availabilityFeature: Feature,
  ) extends ModelClient.Builder {
    override def build(clientId: ClientId, serviceIdentifier: ServiceIdentifier) =
      new DeepbirdClient(
        mkFinagleClient(
          destination,
          name,
          timeout,
          clientId,
          serviceIdentifier,
        ),
        name,
        availabilityFeature,
      )
  }

  def mkFinagleClient(
    destination: String,
    name: String,
    timeout: Duration,
    clientId: ClientId,
    serviceIdentifier: ServiceIdentifier,
  ): DeepbirdPredictionService.MethodPerEndpoint = {
    val stackClient = ThriftMux.client
      .withLabel(name)
      .withClientId(clientId)
      .withMutualTls(serviceIdentifier)

    val servicePerEndpoint: DeepbirdPredictionService.ServicePerEndpoint =
      stackClient
        .methodBuilder(destination)
        .idempotent(Tunables.deepbirdMaxExtraLoad)
        .withTimeoutPerRequest(timeout)
        .withRetryForClassifier(
          ResponseClassifier.RetryOnChannelClosed.orElse(ResponseClassifier.RetryOnTimeout))
        .servicePerEndpoint[DeepbirdPredictionService.ServicePerEndpoint]

    DeepbirdPredictionService.MethodPerEndpoint(servicePerEndpoint)
  }
}
