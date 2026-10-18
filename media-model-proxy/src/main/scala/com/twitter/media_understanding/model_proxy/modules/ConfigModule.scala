package com.twitter.media_understanding.model_proxy.modules

import com.twitter.media_understanding.common.modules.ConfigProviderModule
import com.twitter.media_understanding.model_proxy.config.Config
import com.twitter.server.AdminHttpServer

case class ConfigModule(server: AdminHttpServer) extends ConfigProviderModule[Config](server) {}
