package com.twitter.simclustersann

import com.google.inject.Module
import com.twitter.finagle.Filter
import com.twitter.finagle.ListeningServer
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finatra.annotations.DarkTrafficFilterType
import com.twitter.finatra.decider.modules.DeciderModule
import com.twitter.finatra.mtls.thriftmux.Mtls
import com.twitter.finatra.thrift.ThriftServer
import com.twitter.finatra.thrift.filters._
import com.twitter.finatra.thrift.routing.ThriftRouter
import com.twitter.inject.annotations.Flags
import com.twitter.inject.thrift.modules.ThriftClientIdModule
import com.twitter.relevance_platform.common.exceptions._
import com.twitter.relevance_platform.common.filters.ClientStatsFilter
import com.twitter.relevance_platform.common.filters.DarkTrafficFilterModule
import com.twitter.simclustersann.common.FlagNames.DisableWarmup
import com.twitter.simclustersann.controllers.GetTweetCandidatesGrpcController
import com.twitter.simclustersann.controllers.SimClustersANNController
import com.twitter.simclustersann.exceptions.InvalidRequestForSimClustersAnnVariantExceptionMapper
import com.twitter.simclustersann.modules._
import com.twitter.simclustersann.thriftscala.SimClustersANNService

object SimClustersAnnServerMain extends SimClustersAnnServer

class SimClustersAnnServer extends ThriftServer with Mtls {
  flag(
    name = DisableWarmup,
    default = false,
    help = "If true, no warmup will be run."
  )

  private val grpcPortFlag = flag(
    name = "grpc.port",
    default = 9992,
    help =
      "Port for gRPC (GenericThriftService.GetRequest with Thrift Query / GetTweetCandidates.Result payloads). 0 disables."
  )

  override val name = "simclusters-ann-server"

  override val modules: Seq[Module] = Seq(
    CacheModule,
    SquaredL2NormStoreModule,
    ClusterDetailsModule,
    ServiceNameMapperModule,
    ClusterConfigMapperModule,
    ClusterConfigModule,
    ClusterTweetIndexProviderModule,
    DeciderModule,
    EmbeddingStoreModule,
    FlagsModule,
    FuturePoolProvider,
    RateLimiterModule,
    SimClustersANNCandidateSourceModule,
    StratoClientProviderModule,
    ThriftClientIdModule,
    new CustomMtlsThriftWebFormsModule[SimClustersANNService.MethodPerEndpoint](this),
    new DarkTrafficFilterModule[SimClustersANNService.ReqRepServicePerEndpoint]()
  )

  def configureThrift(router: ThriftRouter): Unit = {
    router
      .filter[LoggingMDCFilter]
      .filter[TraceIdMDCFilter]
      .filter[ThriftMDCFilter]
      .filter[ClientStatsFilter]
      .filter[ExceptionMappingFilter]
      .filter[Filter.TypeAgnostic, DarkTrafficFilterType]
      .exceptionMapper[InvalidRequestForSimClustersAnnVariantExceptionMapper]
      .exceptionMapper[DeadlineExceededExceptionMapper]
      .exceptionMapper[UnhandledExceptionMapper]
      .add[SimClustersANNController]
  }

  override protected def warmup(): Unit = {
    if (!injector.instance[Boolean](Flags.named(DisableWarmup))) {
      handle[SimclustersAnnWarmupHandler]()
    }
  }

  override protected def postWarmup(): Unit = {
    super.postWarmup()
    val port = grpcPortFlag()
    if (port > 0) {
      try {
        val thriftController = injector.instance[SimClustersANNController]
        val grpcImpl = new GetTweetCandidatesGrpcController(thriftController, statsReceiver)
        val grpcServer: ListeningServer = GetTweetCandidatesGrpcController.startListeningServer(
          grpcImpl,
          port,
          injector.instance[ServiceIdentifier],
          statsReceiver
        )
        closeOnExit(grpcServer)
        info(s"gRPC server listening for GenericThriftService.GetRequest on port $port")
      } catch {
        case e: Exception =>
          warn(s"Failed to start gRPC server on port $port: ${e.getMessage}")
      }
    }
  }
}
