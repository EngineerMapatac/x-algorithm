package com.twitter.simclustersann.controllers

import com.google.protobuf.ByteString
import com.twitter.finagle.Http
import com.twitter.finagle.ListeningServer
import com.twitter.finagle.grpc.FinagleServerBuilder
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.mtls.server.MtlsStackServer._
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.scrooge.BinaryThriftStructSerializer
import com.twitter.simclustersann.thriftscala.Query
import com.twitter.simclustersann.thriftscala.SimClustersANNService.GetTweetCandidates
import com.x.grpc_proto.generic_thrift_grpc.GenericThriftRequest
import com.x.grpc_proto.generic_thrift_grpc.GenericThriftResponse
import com.x.grpc_proto.generic_thrift_grpc.GenericThriftServiceGrpc
import io.grpc.Status
import io.grpc.stub.StreamObserver

class GetTweetCandidatesGrpcController(
  thriftController: SimClustersANNController,
  statsReceiver: StatsReceiver)
    extends GenericThriftServiceGrpc.GenericThriftServiceImplBase {

  private[this] val requestSerializer = BinaryThriftStructSerializer(Query)
  private[this] val responseSerializer = BinaryThriftStructSerializer(GetTweetCandidates.Result)

  private[this] val getRequestCounter = statsReceiver.counter("grpc", "get_request")
  private[this] val getRequestSuccessCounter =
    statsReceiver.counter("grpc", "get_request_success")
  private[this] val errorCounter = statsReceiver.counter("grpc", "errors")

  private def deserializeRequestPayload(
    request: GenericThriftRequest
  ): Option[GetTweetCandidates.Args] = {
    val payload = request.getPayload
    if (payload.isEmpty) None
    else
      try Some(GetTweetCandidates.Args(requestSerializer.fromBytes(payload.toByteArray)))
      catch { case _: Exception => None }
  }

  override def getRequest(
    request: GenericThriftRequest,
    responseObserver: StreamObserver[GenericThriftResponse]
  ): Unit = {
    getRequestCounter.incr()
    deserializeRequestPayload(request) match {
      case None =>
        errorCounter.incr()
        responseObserver.onError(
          Status.INVALID_ARGUMENT.withDescription("Invalid request payload").asException())
      case Some(args) =>
        thriftController
          .processGetTweetCandidates(args).onSuccess { candidates =>
            try {
              val thriftResult = GetTweetCandidates.Result(success = Some(candidates))
              val bytes = responseSerializer.toBytes(thriftResult)
              responseObserver.onNext(
                GenericThriftResponse.newBuilder().setPayload(ByteString.copyFrom(bytes)).build())
              responseObserver.onCompleted()
              getRequestSuccessCounter.incr()
            } catch {
              case e: Exception =>
                errorCounter.incr()
                responseObserver.onError(
                  Status.INTERNAL
                    .withDescription(s"Thrift response serialization failed: ${e.getMessage}")
                    .asException())
            }
          }.onFailure { throwable =>
            errorCounter.incr()
            responseObserver.onError(
              Status.INTERNAL
                .withDescription(s"${throwable.getClass.getName}: ${throwable.getMessage}")
                .asException())
          }
    }
  }
}

object GetTweetCandidatesGrpcController {

  def startListeningServer(
    serverImpl: GetTweetCandidatesGrpcController,
    port: Int,
    serviceIdentifier: ServiceIdentifier,
    statsReceiver: StatsReceiver
  ): ListeningServer = {
    val httpServer = Http.server
      .withMutualTls(serviceIdentifier)
      .withStatsReceiver(statsReceiver.scope("grpc"))
    val server = FinagleServerBuilder
      .forPort(port)
      .httpServer(httpServer)
      .addService(serverImpl)
      .build()
    server.start()
    server
  }
}
