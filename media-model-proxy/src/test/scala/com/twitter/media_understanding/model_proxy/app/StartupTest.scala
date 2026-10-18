package com.twitter.media_understanding.model_proxy.app

import com.google.inject.Stage
import com.twitter.finatra.mtls.thriftmux.EmbeddedMtlsThriftServer
import com.twitter.inject.server.{FeatureTest => FinatraFeatureTest}
import org.specs2.mock.Mockito

class StartupTest extends FinatraFeatureTest with Mockito {
  val server = new EmbeddedMtlsThriftServer(
    twitterServer = new MediaModelProxyThriftServer(),
    stage = Stage.DEVELOPMENT,
    flags = Map(
      "com.twitter.server.wilyns.disable" -> "true",
      "config.overlay" -> "",
      "dtab.add" -> "/$/inet => /$/nil; /zk => /$/nil",
      "decider.base" -> "/test_decider_base.yml"
    )
  )

  test("Server#startup") {
    server.assertHealthy()
  }
}
