package com.twitter.simclusters_v2.stores

import com.twitter.scalding_internal.multiformat.format.keyval.KeyValInjection.Long2BigEndian
import com.twitter.scalding_internal.multiformat.format.keyval.KeyValInjection.ScalaBinaryThrift
import com.twitter.storage.client.manhattan.kv.ManhattanKVClientMtlsParams
import com.twitter.storehaus.ReadableStore
import com.twitter.storehaus_internal.manhattan.Apollo
import com.twitter.storehaus_internal.manhattan.ManhattanRO
import com.twitter.storehaus_internal.manhattan.ManhattanROConfig
import com.twitter.storehaus_internal.util.ApplicationID
import com.twitter.storehaus_internal.util.DatasetName
import com.twitter.storehaus_internal.util.HDFSPath
import com.twitter.wtf.candidate.thriftscala.CandidateSeq

object WtfMbcgStore {

  val appId = "recos_platform_apollo"

  implicit val keyInj = Long2BigEndian
  implicit val valInj = ScalaBinaryThrift(CandidateSeq)

  def getWtfMbcgStore(
    mhMtlsParams: ManhattanKVClientMtlsParams,
    datasetName: String
  ): ReadableStore[Long, CandidateSeq] = {
    ManhattanRO.getReadableStoreWithMtls[Long, CandidateSeq](
      ManhattanROConfig(
        HDFSPath(""),
        ApplicationID(appId),
        DatasetName(datasetName),
        Apollo
      ),
      mhMtlsParams
    )
  }
}
