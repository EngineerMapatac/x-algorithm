package com.twitter.simclusters_v2.summingbird.stores
import com.twitter.simclusters_v2.thriftscala.ClustersUserIsInterestedIn
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingId
import com.twitter.storage.client.manhattan.kv.ManhattanKVClient
import com.twitter.storage.client.manhattan.kv.ManhattanKVClientMtlsParams
import com.twitter.storage.client.manhattan.kv.ManhattanKVEndpointBuilder
import com.twitter.storage.client.manhattan.kv.impl.Component
import com.twitter.storage.client.manhattan.kv.impl.DescriptorP1L0
import com.twitter.storage.client.manhattan.kv.impl.KeyDescriptor
import com.twitter.storage.client.manhattan.kv.impl.ValueDescriptor
import com.twitter.storehaus.ReadableStore
import com.twitter.storehaus_internal.manhattan.ManhattanCluster
import com.twitter.storehaus_internal.manhattan.Adama
import com.twitter.storage.client.manhattan.bijections.Bijections.BinaryScalaInjection
import com.twitter.storage.client.manhattan.kv.Guarantee
import com.twitter.conversions.DurationOps._
import com.twitter.simclusters_v2.thriftscala.InternalId
import com.twitter.stitch.Stitch
import com.twitter.storage.client.manhattan.bijections.Bijections.LongInjection
import com.twitter.util.Future

class SimClustersManhattanReadableStoreForReadWriteDataset(
  appId: String,
  datasetName: String,
  label: String,
  mtlsParams: ManhattanKVClientMtlsParams,
  manhattanCluster: ManhattanCluster = Adama)
    extends ReadableStore[SimClustersEmbeddingId, ClustersUserIsInterestedIn] {
  val destName = manhattanCluster.wilyName
  val endPoint = ManhattanKVEndpointBuilder(ManhattanKVClient(appId, destName, mtlsParams, label))
    .defaultGuarantee(Guarantee.SoftDcReadMyWrites)
    .build()

  val keyDesc = KeyDescriptor(Component(LongInjection), Component()).withDataset(datasetName)
  val valueDesc = ValueDescriptor(BinaryScalaInjection(ClustersUserIsInterestedIn))

  override def get(
    embeddingId: SimClustersEmbeddingId
  ): Future[Option[ClustersUserIsInterestedIn]] = {
    embeddingId match {
      case SimClustersEmbeddingId(theEmbeddingType, theModelVersion, InternalId.UserId(userId)) =>
        val populatedKey: DescriptorP1L0.FullKey[Long] = keyDesc.withPkey(userId)
        val mhValue = Stitch.run(endPoint.get(populatedKey, valueDesc))
        mhValue.map {
          case Some(x) => Option(x.contents)
          case _ => None
        }
      case _ => Future.None
    }
  }
}
