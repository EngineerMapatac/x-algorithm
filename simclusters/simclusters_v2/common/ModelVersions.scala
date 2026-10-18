package com.twitter.simclusters_v2.common

import com.twitter.simclusters_v2.thriftscala.ModelVersion

object ModelVersions {

  val Model20M145KDec11 = "20M_145K_dec11"
  val Model20M145KUpdated = "20M_145K_updated"
  val Model20M145K2020 = "20M_145K_2020"

  object Enum extends Enumeration {
    val Model20M145K2020, Model20M145KUpdated: Value = Value
    val enumToSimClustersModelVersionMap: Map[Enum.Value, ModelVersion] = Map(
      Model20M145K2020 -> ModelVersion.Model20m145k2020,
      Model20M145KUpdated -> ModelVersion.Model20m145kUpdated
    )
  }

  private val StringToThriftModelVersions: Map[String, ModelVersion] =
    Map(
      Model20M145KDec11 -> ModelVersion.Model20m145kDec11,
      Model20M145KUpdated -> ModelVersion.Model20m145kUpdated,
      Model20M145K2020 -> ModelVersion.Model20m145k2020
    )

  private val ThriftModelVersionToStrings = StringToThriftModelVersions.map(_.swap)

  val AllModelVersions: Set[String] = StringToThriftModelVersions.keySet

  def toModelVersionOption(modelVersionStr: String): Option[ModelVersion] = {
    StringToThriftModelVersions.get(modelVersionStr)
  }

  implicit def toModelVersion(modelVersionStr: String): ModelVersion = {
    StringToThriftModelVersions(modelVersionStr)
  }

  implicit def toKnownForModelVersion(modelVersion: ModelVersion): String = {
    ThriftModelVersionToStrings(modelVersion)
  }

}
