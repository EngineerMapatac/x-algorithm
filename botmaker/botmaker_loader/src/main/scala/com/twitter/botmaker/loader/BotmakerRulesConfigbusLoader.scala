package com.twitter.botmaker.loader

import com.twitter.app.GlobalFlag
import com.twitter.botmaker.common.SerializerUtil
import com.twitter.botmaker.rule.thriftscala.BotMakerRulesPackage
import com.twitter.botmaker.util.ThriftUtil
import com.twitter.configbus.client.ConfigSource
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.io.Buf

object BotmakerRulesConfigbusLoader {

  def deserializeJson(buf: Buf): BotMakerRulesPackage = {
    val bytes = Buf.ByteArray.Owned.extract(buf)
    SerializerUtil.fromJson(BotMakerRulesPackage, bytes)
  }

  val defaultIfNotFound: Option[BotMakerRulesPackage] = if (mustHavePackage()) {
    None
  } else {
    Some(BotMakerRulesPackage())
  }
}

object mustHavePackage
    extends GlobalFlag[Boolean](
      default = true,
      help =
        "When set to true, rules package must be successfully loaded from configbus or service " +
          "will fail to start.")

class BotmakerRulesConfigbusLoader(
  loaderName: String,
  configSource: ConfigSource,
  configFileName: String,
  statsReceiver: StatsReceiver)
    extends ConfigbusLoader[BotMakerRulesPackage](
      loaderName,
      configSource,
      configFileName,
      statsReceiver,
      buf => BotmakerRulesConfigbusLoader.deserializeJson(buf),
      defaultIfNotFound = BotmakerRulesConfigbusLoader.defaultIfNotFound,
      loadedInformationForLogging = value =>
        s"${value.rules.size} and ${value.derivedFeatures.size} derived features"
    )
