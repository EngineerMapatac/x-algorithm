package com.twitter.simclusters_v2.summingbird.stores

import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.finagle.thrift.ClientId
import com.twitter.gizmoduck.thriftscala.LookupContext
import com.twitter.gizmoduck.thriftscala.QueryFields
import com.twitter.gizmoduck.thriftscala.User
import com.twitter.gizmoduck.thriftscala.UserService
import com.twitter.hermit.store.common.ObservedReadableStore
import com.twitter.hermit.store.gizmoduck.{GizmoduckUserStore => UserStore}
import com.twitter.finagle.thrift.RichClientParam
import com.twitter.gizmoduck.client.GizmoduckClientBuilder
import com.twitter.servo.client.DefaultFinagleClientBuilder
import com.twitter.servo.client.Environment
import com.twitter.storehaus.ReadableStore

object GizmoduckUserStore {
  def gizmoduckReadableStore(
    serviceIdentifier: ServiceIdentifier,
    clientId: ClientId,
    statsReceiver: StatsReceiver
  ): ReadableStore[Long, User] = {
    val gizmoduckService =
      GizmoduckClientBuilder(
        finagleClientBuilder = DefaultFinagleClientBuilder(clientId),
        environment = Environment.Production,
        serviceIdentifier = serviceIdentifier
      ).build()

    val gizmoduckClient = new UserService.FinagledClient(
      gizmoduckService,
      RichClientParam(
        serviceName = GizmoduckClientBuilder.DefaultName,
        clientStats = statsReceiver
      ))

    ObservedReadableStore(
      UserStore(
        gizmoduckClient,
        Set(QueryFields.Safety),
        LookupContext()
      )
    )(statsReceiver.scope("GizmoduckUserStore"))
  }
}
