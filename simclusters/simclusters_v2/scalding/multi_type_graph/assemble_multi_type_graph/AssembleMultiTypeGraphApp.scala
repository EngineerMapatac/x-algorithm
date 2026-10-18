package com.twitter.simclusters_v2.scalding
package multi_type_graph.assemble_multi_type_graph

import com.twitter.dal.client.dataset.KeyValDALDataset
import com.twitter.dal.client.dataset.SnapshotDALDataset
import com.twitter.scalding.Days
import com.twitter.scalding.Duration
import com.twitter.scalding.RichDate
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.thriftscala.LeftNode
import com.twitter.simclusters_v2.thriftscala.RightNodeTypeStruct
import com.twitter.simclusters_v2.thriftscala.RightNodeWithEdgeWeightList
import com.twitter.simclusters_v2.thriftscala.NounWithFrequencyList
import com.twitter.simclusters_v2.thriftscala.MultiTypeGraphEdge
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import com.twitter.simclusters_v2.hdfs_sources._

object AssembleMultiTypeGraphAdhocApp extends AssembleMultiTypeGraphBaseApp with AdhocExecutionApp {
  override val isAdhoc: Boolean = true
  override val truncatedMultiTypeGraphMHOutputPath: String = "truncated_graph_mh"
  override val topKRightNounsMHOutputPath: String = "top_k_right_nouns_mh"
  override val fullMultiTypeGraphThriftOutputPath: String = "full_graph_thrift"
  override val truncatedMultiTypeGraphKeyValDataset: KeyValDALDataset[
    KeyVal[LeftNode, RightNodeWithEdgeWeightList]
  ] = TruncatedMultiTypeGraphAdhocScalaDataset
  override val topKRightNounsKeyValDataset: KeyValDALDataset[
    KeyVal[RightNodeTypeStruct, NounWithFrequencyList]
  ] = TopKRightNounsAdhocScalaDataset
  override val fullMultiTypeGraphSnapshotDataset: SnapshotDALDataset[MultiTypeGraphEdge] =
    FullMultiTypeGraphAdhocScalaDataset
}

object AssembleMultiTypeGraphBatchApp
    extends AssembleMultiTypeGraphBaseApp
    with ScheduledExecutionApp {
  override val isAdhoc: Boolean = false
  override val truncatedMultiTypeGraphMHOutputPath: String = "truncated_graph_mh"
  override val topKRightNounsMHOutputPath: String = "top_k_right_nouns_mh"
  override val fullMultiTypeGraphThriftOutputPath: String = "full_graph_thrift"
  override val truncatedMultiTypeGraphKeyValDataset: KeyValDALDataset[
    KeyVal[LeftNode, RightNodeWithEdgeWeightList]
  ] = TruncatedMultiTypeGraphScalaDataset
  override val topKRightNounsKeyValDataset: KeyValDALDataset[
    KeyVal[RightNodeTypeStruct, NounWithFrequencyList]
  ] = TopKRightNounsScalaDataset
  override val fullMultiTypeGraphSnapshotDataset: SnapshotDALDataset[MultiTypeGraphEdge] =
    FullMultiTypeGraphScalaDataset
  override val firstTime: RichDate = RichDate("2021-08-21")
  override val batchIncrement: Duration = Days(7)
}
