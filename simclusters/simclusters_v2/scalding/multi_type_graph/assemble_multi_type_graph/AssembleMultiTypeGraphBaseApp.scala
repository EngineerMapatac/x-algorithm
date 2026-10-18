package com.twitter.simclusters_v2.scalding
package multi_type_graph.assemble_multi_type_graph

import com.twitter.dal.client.dataset.KeyValDALDataset
import com.twitter.dal.client.dataset.SnapshotDALDataset
import com.twitter.scalding.Execution
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DALWrite.D
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.scalding.common.TypedRichPipe.typedPipeToRichPipe
import com.twitter.simclusters_v2.scalding.common.Util
import com.twitter.simclusters_v2.thriftscala.LeftNode
import com.twitter.simclusters_v2.thriftscala.Noun
import com.twitter.simclusters_v2.thriftscala.NounWithFrequency
import com.twitter.simclusters_v2.thriftscala.NounWithFrequencyList
import com.twitter.simclusters_v2.thriftscala.RightNodeType
import com.twitter.simclusters_v2.thriftscala.RightNodeTypeStruct
import com.twitter.simclusters_v2.thriftscala.RightNodeWithEdgeWeight
import com.twitter.simclusters_v2.thriftscala.RightNodeWithEdgeWeightList
import com.twitter.simclusters_v2.thriftscala.MultiTypeGraphEdge
import com.twitter.wtf.scalding.jobs.common.DateRangeExecutionApp
import java.util.TimeZone

trait AssembleMultiTypeGraphBaseApp extends DateRangeExecutionApp {
  val truncatedMultiTypeGraphKeyValDataset: KeyValDALDataset[
    KeyVal[LeftNode, RightNodeWithEdgeWeightList]
  ]
  val topKRightNounsKeyValDataset: KeyValDALDataset[
    KeyVal[RightNodeTypeStruct, NounWithFrequencyList]
  ]
  val fullMultiTypeGraphSnapshotDataset: SnapshotDALDataset[MultiTypeGraphEdge]
  val isAdhoc: Boolean
  val truncatedMultiTypeGraphMHOutputPath: String
  val topKRightNounsMHOutputPath: String
  val fullMultiTypeGraphThriftOutputPath: String

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    import Config._
    import AssembleMultiTypeGraph._

    val numKeysInTruncatedGraph = Stat("num_keys_truncated_mts")
    val numKeysInTopKNounsGraph = Stat("num_keys_topk_nouns_mts")

    val fullGraph: TypedPipe[(LeftNode, RightNodeWithEdgeWeight)] =
      getFullGraph().count("num_entries_full_graph")

    val topKRightNodes: TypedPipe[(RightNodeType, Seq[(Noun, Double)])] =
      getTopKRightNounsWithFrequencies(
        fullGraph,
        TopKConfig,
        GlobalDefaultMinFrequencyOfRightNodeType)

    val truncatedGraph: TypedPipe[(LeftNode, RightNodeWithEdgeWeight)] =
      getTruncatedGraph(fullGraph, topKRightNodes).count("num_entries_truncated_graph")

    val truncatedGraphKeyedBySrc: TypedPipe[(LeftNode, RightNodeWithEdgeWeightList)] =
      truncatedGraph
        .map {
          case (LeftNode.UserId(userId), rightNodeWithWeight) =>
            userId -> List(rightNodeWithWeight)
        }
        .sumByKey
        .map {
          case (userId, rightNodeWithWeightList) =>
            (LeftNode.UserId(userId), RightNodeWithEdgeWeightList(rightNodeWithWeightList))
        }

    val topKNounsKeyedByType: TypedPipe[(RightNodeTypeStruct, NounWithFrequencyList)] =
      topKRightNodes
        .map {
          case (rightNodeType, rightNounsWithScoresList) =>
            val nounsListWithFrequency: Seq[NounWithFrequency] = rightNounsWithScoresList
              .map {
                case (noun, aggregatedFrequency) =>
                  NounWithFrequency(noun, aggregatedFrequency)
              }
            (RightNodeTypeStruct(rightNodeType), NounWithFrequencyList(nounsListWithFrequency))
        }

    val truncatedGraphTsvExec: Execution[Unit] =
      truncatedGraphKeyedBySrc.writeExecution(
        TypedTsv[(LeftNode, RightNodeWithEdgeWeightList)](AdhocRootPrefix + "truncated_graph_tsv"))

    val truncatedGraphDALExec: Execution[Unit] = truncatedGraphKeyedBySrc
      .map {
        case (leftNode, rightNodeWithWeightList) =>
          numKeysInTruncatedGraph.inc()
          KeyVal(leftNode, rightNodeWithWeightList)
      }
      .writeDALVersionedKeyValExecution(
        truncatedMultiTypeGraphKeyValDataset,
        D.Suffix(
          (if (!isAdhoc)
             RootPath
           else
             AdhocRootPrefix)
            + truncatedMultiTypeGraphMHOutputPath),
        ExplicitEndTime(dateRange.`end`)
      )

    val topKNounsTsvExec: Execution[Unit] =
      topKNounsKeyedByType.writeExecution(
        TypedTsv[(RightNodeTypeStruct, NounWithFrequencyList)](
          AdhocRootPrefix + "top_k_right_nouns_tsv"))

    val topKNounsDALExec: Execution[Unit] = topKNounsKeyedByType
      .map {
        case (engagementType, rightList) =>
          val rightListMH =
            NounWithFrequencyList(rightList.nounWithFrequencyList.take(TopKRightNounsForMHDump))
          numKeysInTopKNounsGraph.inc()
          KeyVal(engagementType, rightListMH)
      }
      .writeDALVersionedKeyValExecution(
        topKRightNounsKeyValDataset,
        D.Suffix(
          (if (!isAdhoc)
             RootPath
           else
             AdhocRootPrefix)
            + topKRightNounsMHOutputPath),
        ExplicitEndTime(dateRange.`end`)
      )

    val fullGraphDALExec: Execution[Unit] = fullGraph
      .map {
        case (leftNode, rightNodeWithWeight) =>
          MultiTypeGraphEdge(leftNode, rightNodeWithWeight)
      }.writeDALSnapshotExecution(
        fullMultiTypeGraphSnapshotDataset,
        D.Daily,
        D.Suffix(
          (if (!isAdhoc)
             RootThriftPath
           else
             AdhocRootPrefix)
            + fullMultiTypeGraphThriftOutputPath),
        D.Parquet,
        dateRange.`end`
      )

    if (isAdhoc) {
      Util.printCounters(
        Execution
          .zip(
            truncatedGraphTsvExec,
            topKNounsTsvExec,
            truncatedGraphDALExec,
            topKNounsDALExec,
            fullGraphDALExec).unit)
    } else {
      Util.printCounters(
        Execution.zip(truncatedGraphDALExec, topKNounsDALExec, fullGraphDALExec).unit)
    }

  }
}
