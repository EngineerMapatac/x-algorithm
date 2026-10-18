package com.twitter.botmaker.botmaker_package

import com.twitter.io.Buf
import java.io.ByteArrayInputStream
import java.io.InputStreamReader
import java.nio.charset.StandardCharsets
import org.yaml.snakeyaml.Yaml
import org.yaml.snakeyaml.constructor.SafeConstructor
import java.util.{Map => JMap}
import java.util.{LinkedHashMap => JLinkedHashMap}

final case class BotMakerPackageDetails(
  versionNumber: Option[Long])

object BotMakerPackageDetails {

  object Yaml {
    private[this] val yamlSerdes: ThreadLocal[Yaml] =
      ThreadLocal.withInitial(() => new Yaml(new SafeConstructor()))

    def fromYaml(buf: Buf): BotMakerPackageDetails = {
      val bytes: Array[Byte] = Buf.ByteArray.Owned.extract(buf)
      val theMap: JMap[String, String] = yamlSerdes.get.load(
        new InputStreamReader(new ByteArrayInputStream(bytes), StandardCharsets.UTF_8))

      val versionNumber = Option(theMap.get("version")).map(_.toLong)

      BotMakerPackageDetails(versionNumber)
    }

    def toYaml(packageDetails: BotMakerPackageDetails): Buf = {
      val yamlMap: JMap[String, String] = new JLinkedHashMap[String, String]
      packageDetails.versionNumber.foreach { versionNumber =>
        yamlMap.put("version", String.valueOf(versionNumber))
      }

      val str: String = yamlSerdes.get.dumpAsMap(yamlMap)
      Buf.Utf8(str)
    }
  }

  def fromYaml(buf: Buf): BotMakerPackageDetails = {
    Yaml.fromYaml(buf)
  }

  def toYaml(details: BotMakerPackageDetails): Buf = {
    Yaml.toYaml(details)
  }
}
