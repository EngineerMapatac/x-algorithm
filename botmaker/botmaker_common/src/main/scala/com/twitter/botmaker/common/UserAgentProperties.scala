package com.twitter.botmaker.common

import org.uaparser.scala.Parser

case class UserAgentProperties(
  isMobile: Boolean,
  isTablet: Boolean,
  isDesktop: Boolean,
  isiPad: Boolean,
  isiPhone: Boolean,
  isAndroid: Boolean,
  isIE: Boolean,
  isOpera: Boolean,
  isFirefox: Boolean,
  isChrome: Boolean,
  isSafari: Boolean,
  isWindows: Boolean,
  isLinux: Boolean,
  isMac: Boolean,
  isChromeOS: Boolean)

object UserAgentProperties {
  private val uaParser = Parser.default
  private val mobileRegex = containsRegex("mobile")

  def apply(userAgent: String): UserAgentProperties = {
    val parsedUserAgent = uaParser.parse(userAgent)
    val device = parsedUserAgent.device.family
    val os = parsedUserAgent.os.family
    val browser = parsedUserAgent.userAgent.family

    val containsMobileText = userAgent.matches(mobileRegex)

    val isChrome = browser.contains("Chrome")
    val isFirefox = browser.contains("Firefox")
    val isSafari = browser.contains("Safari")
    val isOpera = browser.contains("Opera")
    val isIE = browser.contains("IE")

    val isWindows = Seq(device, os).contains("Windows")
    val isWindowsPhone = os.contains("Windows Phone")
    val isLinux = Seq(device, os).contains("Linux")
    val isMac = Seq(device, os).exists(family => family.contains("Mac"))
    val isBada = os.contains("Bada")
    val isBlackberry = Seq(os, device).exists(family => family.contains("Blackberry"))
    val isSamsung = device.contains("Samsung")
    val isiPod = device == "iPod"
    val isiPad = device == "iPad"
    val isiPhone = device == "iPhone"
    val isChromeOS = os.contains("Chrome")
    val isAndroid = os.contains("Android")
    val isAndroidTablet = isAndroid && !containsMobileText
    val isKindleFire = device.contains("Kindle")

    val isTablet = isiPad || isAndroidTablet || isKindleFire
    val isMobile =
      containsMobileText || isAndroid || isSamsung || isiPad || isiPod || isiPhone || isBada || isBlackberry || isWindowsPhone
    val isDesktop = !isMobile && (isWindows || isLinux || isMac || isChromeOS)

    UserAgentProperties(
      isDesktop = isDesktop,
      isMobile = isMobile,
      isTablet = isTablet,
      isiPad = isiPad,
      isiPhone = isiPhone,
      isAndroid = isAndroid,
      isIE = isIE,
      isOpera = isOpera,
      isFirefox = isFirefox,
      isChrome = isChrome,
      isSafari = isSafari,
      isWindows = isWindows,
      isLinux = isLinux,
      isMac = isMac,
      isChromeOS = isChromeOS
    )
  }

  private def containsRegex(containedText: String) = s"(?i).*($containedText).*"

}
