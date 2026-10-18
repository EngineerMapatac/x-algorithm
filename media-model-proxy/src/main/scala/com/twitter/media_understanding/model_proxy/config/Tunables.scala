package com.twitter.media_understanding.model_proxy.config

import com.twitter.finagle.tunable.StandardTunableMap
import com.twitter.util.tunable.Tunable
import com.twitter.util.tunable.{TunableMap => UtilTunableMap}

object Tunables {
  private[model_proxy] final val TunableId = "cortex_media_annotator"

  private[this] val tunables: UtilTunableMap = StandardTunableMap(TunableId)

  private[this] val deepbirdMaxExtraLoadTunable: Tunable[Double] = tunables(
    UtilTunableMap.Key[Double]("com.twitter.media_understanding.model_proxy.deepbirdMaxExtraLoad")
  )

  private[this] val blobStoreMaxExtraLoadTunable: Tunable[Double] = tunables(
    UtilTunableMap.Key[Double]("com.twitter.media_understanding.model_proxy.blobStoreMaxExtraLoad")
  )

  def deepbirdMaxExtraLoad: Tunable[Double] = deepbirdMaxExtraLoadTunable.map(_.doubleValue())

  def blobStoreMaxExtraLoad: Tunable[Double] = blobStoreMaxExtraLoadTunable.map(_.doubleValue())
}
