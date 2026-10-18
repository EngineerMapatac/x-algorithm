package com.twitter.botmaker.common

import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit1
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.common_internal.analytics.twitter_client_user_agent_parser.UserAgentParser

object UserAgent extends FunctionUnit1[TwitterRuntime, String, Map[String, String]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set()
  override def description =
    "Use UserAgentParser to parse user agent string and output a map"
  override def arguments: Seq[String] =
    Seq[String]("user agent string")
  override def examples: Seq[String] =
    Seq[String](
      "UserAgent(\"twidge v1.1.0; Haskell. GHC\",)"
    )

  override def evaluate(
    context: com.twitter.botmaker.Context[TwitterRuntime],
    userAgentString: String
  ): Map[String, String] = {
    val map = collection.mutable.Map[String, String]()
    UserAgentParser.parse(userAgentString) match {
      case None =>
      case Some(userAgent) =>
        if (userAgent.clientName.isDefined) {
          map += ("ClientName" -> userAgent.clientName.get)
        }
        if (userAgent.clientVersion.isDefined) {
          map += ("ClientVersion" -> userAgent.clientVersion.get)
        }
        if (userAgent.clientVersionCode.isDefined) {
          map += ("ClientVersionCode" -> userAgent.clientVersionCode.get)
        }
        if (userAgent.model.isDefined) {
          map += ("Model" -> userAgent.model.get)
        }
        if (userAgent.sdkVersion.isDefined) {
          map += ("SDKVersion" -> userAgent.sdkVersion.get)
        }
        if (userAgent.manufacturer.isDefined) {
          map += ("Manufacturer" -> userAgent.manufacturer.get)
        }
        if (userAgent.device.isDefined) {
          map += ("Device" -> userAgent.device.get)
        }
        if (userAgent.clientSource.isDefined) {
          map += ("ClientSource" -> userAgent.clientSource.get)
        }
        if (userAgent.wifi.isDefined) {
          map += ("WiFi" -> userAgent.wifi.get)
        }
        if (userAgent.yearClass.isDefined) {
          map += ("YearClass" -> userAgent.yearClass.get)
        }

        map += ("IsTwitterClient" -> userAgent.isTwitterClient.toString.toUpperCase)
        map += ("IsDogFood" -> userAgent.isDogfood.toString.toUpperCase)
        map += ("IsBeta" -> userAgent.isBeta.toString.toUpperCase)
        val properties = UserAgentProperties(userAgentString)
        map += ("IsMobile" -> formatBool(properties.isMobile))
        map += ("IsTablet" -> formatBool(properties.isTablet))
        map += ("IsDesktop" -> formatBool(properties.isDesktop))
        map += ("IsiPad" -> formatBool(properties.isiPad))
        map += ("IsiPhone" -> formatBool(properties.isiPhone))
        map += ("IsAndroid" -> formatBool(properties.isAndroid))
        map += ("IsIE" -> formatBool(properties.isIE))
        map += ("IsOpera" -> formatBool(properties.isOpera))
        map += ("IsFirefox" -> formatBool(properties.isFirefox))
        map += ("IsChrome" -> formatBool(properties.isChrome))
        map += ("IsSafari" -> formatBool(properties.isSafari))
        map += ("IsWindows" -> formatBool(properties.isWindows))
        map += ("IsLinux" -> formatBool(properties.isLinux))
        map += ("IsMac" -> formatBool(properties.isMac))
        map += ("IsChromeOS" -> formatBool(properties.isChromeOS))
    }
    map.toMap
  }

  private def formatBool(bool: Boolean): String = if (bool) "TRUE" else "FALSE"
}
