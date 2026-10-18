package com.twitter.media_understanding.model_proxy.modules

import com.twitter.cortex_media_annotator.thriftscala.CortexMediaAnnotator
import com.twitter.decider.Decider
import com.twitter.decider.RandomRecipient
import com.twitter.finatra.mtls.thriftmux.modules.MtlsClient
import com.twitter.inject.Injector
import com.twitter.inject.thrift.modules.ReqRepDarkTrafficFilterModule
import com.twitter.media_understanding.model_proxy.config.Deciders

class ThriftDiffyProxyModule
    extends ReqRepDarkTrafficFilterModule[CortexMediaAnnotator.ReqRepServicePerEndpoint]
    with MtlsClient {

  override val label: String = "diffy-proxy"

  private val diffyProxyEnabled = flag(
    "diffy_proxy.enabled",
    false,
    "Enabled Diffy Proxy forwarding"
  )

  private def diffyProxyDeciderEnabled(injector: Injector): Boolean = {
    val decider = injector.instance[Decider]
    decider.isAvailable(Deciders.DiffyProxyDarkTrafficRate, Some(RandomRecipient))
  }

  override def enableSampling(injector: Injector): Any => Boolean = { request =>
    diffyProxyDeciderEnabled(injector) && diffyProxyEnabled()
  }

}
