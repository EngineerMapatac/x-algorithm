package com.twitter.simclusters_v2.scalding.optout

import com.twitter.dal.client.dataset.KeyValDALDataset
import com.twitter.dal.client.dataset.SnapshotDALDataset
import com.twitter.scalding.Args
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.Duration
import com.twitter.scalding.Execution
import com.twitter.scalding.RichDate
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.TypedTsv
import com.twitter.scalding.UniqueID
import com.twitter.scalding_internal.dalv2.DALWrite.D
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.common.ClusterId
import com.twitter.simclusters_v2.common.ModelVersions
import com.twitter.simclusters_v2.common.SemanticCoreEntityId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.hdfs_sources._
import com.twitter.simclusters_v2.scalding.inferred_entities.InferredEntities
import com.twitter.simclusters_v2.thriftscala.ClusterType
import com.twitter.simclusters_v2.thriftscala.ClustersUserIsInterestedIn
import com.twitter.simclusters_v2.thriftscala.SemanticCoreEntityWithScore
import com.twitter.simclusters_v2.thriftscala.UserToInterestedInClusters
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import com.twitter.simclusters_v2.scalding.common.TypedRichPipe._
import com.twitter.simclusters_v2.scalding.common.Util
import java.util.TimeZone

object InterestedInOptOut {

  def filterOptedOutInterestedIn(
    interestedInPipe: TypedPipe[(UserId, ClustersUserIsInterestedIn)],
    optedOutEntities: TypedPipe[(UserId, Set[SemanticCoreEntityId])],
    clusterToEntities: TypedPipe[(ClusterId, Seq[SemanticCoreEntityWithScore])]
  ): TypedPipe[(UserId, ClustersUserIsInterestedIn)] = {

    val validInterestedIn = SimClustersOptOutUtil.filterOptedOutClusters(
      userToClusters = interestedInPipe.mapValues(_.clusterIdToScores.keySet.toSeq),
      optedOutEntities = optedOutEntities,
      legibleClusters = clusterToEntities
    )

    interestedInPipe
      .leftJoin(validInterestedIn)
      .mapValues {
        case (originalInterestedIn, validInterestedInOpt) =>
          val validInterestedIn = validInterestedInOpt.getOrElse(Seq()).toSet

          originalInterestedIn.copy(
            clusterIdToScores = originalInterestedIn.clusterIdToScores.filterKeys(validInterestedIn)
          )
      }
      .filter(_._2.clusterIdToScores.nonEmpty)
  }

  def writeInterestedInOutputExecution(
    interestedIn: TypedPipe[(UserId, ClustersUserIsInterestedIn)],
    interestedInDataset: KeyValDALDataset[KeyVal[Long, ClustersUserIsInterestedIn]],
    outputPath: String
  ): Execution[Unit] = {
    interestedIn
      .map { case (k, v) => KeyVal(k, v) }
      .writeDALVersionedKeyValExecution(
        interestedInDataset,
        D.Suffix(outputPath)
      )
  }

  def writeInterestedInThriftOutputExecution(
    interestedIn: TypedPipe[(UserId, ClustersUserIsInterestedIn)],
    modelVersion: String,
    interestedInThriftDatset: SnapshotDALDataset[UserToInterestedInClusters],
    thriftOutputPath: String,
    dateRange: DateRange
  ): Execution[Unit] = {
    interestedIn
      .map {
        case (userId, clusters) =>
          UserToInterestedInClusters(userId, modelVersion, clusters.clusterIdToScores)
      }
      .writeDALSnapshotExecution(
        interestedInThriftDatset,
        D.Daily,
        D.Suffix(thriftOutputPath),
        D.EBLzo(),
        dateRange.end
      )
  }
}

object InterestedInOptOutDailyBatchJob extends ScheduledExecutionApp {

  override def firstTime: RichDate = RichDate("2019-11-24")

  override def batchIncrement: Duration = Days(1)

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {

    val userOptoutEntities =
      SimClustersOptOutUtil
        .getP13nOptOutSources(dateRange.embiggen(Days(4)), ClusterType.InterestedIn)
        .count("num_users_with_optouts")
        .forceToDisk

    val interestedIn2020Pipe = InterestedInSources
      .simClustersRawInterestedIn2020Source(dateRange.embiggen(Days(28)), timeZone)
      .count("num_users_with_2020_interestedin")

    val clusterToEntities = InferredEntities
      .getLegibleEntityEmbeddings(dateRange.prepend(Days(21)), timeZone)
      .count("num_cluster_to_entities")

    val filtered2020InterestedIn = InterestedInOptOut
      .filterOptedOutInterestedIn(interestedIn2020Pipe, userOptoutEntities, clusterToEntities)
      .count("num_users_with_compliant_2020_interestedin")

    val write2020Exec = InterestedInOptOut.writeInterestedInOutputExecution(
      filtered2020InterestedIn,
      SimclustersV2InterestedIn20M145K2020ScalaDataset,
      DataPaths.InterestedIn2020Path
    )

    val write2020ThriftExec = InterestedInOptOut.writeInterestedInThriftOutputExecution(
      filtered2020InterestedIn,
      ModelVersions.Model20M145K2020,
      SimclustersV2UserToInterestedIn20M145K2020ScalaDataset,
      DataPaths.InterestedIn2020ThriftPath,
      dateRange
    )

    val sanityCheck2020Exec = SimClustersOptOutUtil.sanityCheckAndSendEmail(
      oldNumClustersPerUser = interestedIn2020Pipe.map(_._2.clusterIdToScores.size),
      newNumClustersPerUser = filtered2020InterestedIn.map(_._2.clusterIdToScores.size),
      modelVersion = ModelVersions.Model20M145K2020,
      alertEmail = SimClustersOptOutUtil.AlertEmail
    )

    Util.printCounters(
      Execution.zip(write2020Exec, write2020ThriftExec, sanityCheck2020Exec)
    )
  }
}

object InterestedInOptOutAdhocJob extends AdhocExecutionApp {
  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    val outputDir = args("outputDir")

    val interestedInPipe = InterestedInSources
      .simClustersInterestedInUpdatedSource(dateRange, timeZone)
      .count("num_users_with_interestedin")

    val userOptoutEntities: TypedPipe[(UserId, Set[SemanticCoreEntityId])] =
      SimClustersOptOutUtil
        .getP13nOptOutSources(dateRange.embiggen(Days(4)), ClusterType.InterestedIn)
        .count("num_users_with_optouts")

    val clusterToEntities = InferredEntities
      .getLegibleEntityEmbeddings(dateRange, timeZone)
      .count("num_cluster_to_entities")

    val filteredInterestedInPipe = InterestedInOptOut
      .filterOptedOutInterestedIn(
        interestedInPipe,
        userOptoutEntities,
        clusterToEntities
      )
      .count("num_users_with_interestedin_after_optout")

    val output = interestedInPipe
      .join(filteredInterestedInPipe)
      .filter {
        case (userId, (originalInterestedIn, filtered)) =>
          originalInterestedIn.clusterIdToScores != filtered.clusterIdToScores
      }
      .join(userOptoutEntities)
      .map {
        case (userId, ((originalInterestedIn, filtered), optoutEntities)) =>
          Seq(
            "userId=" + userId,
            "originalInterestedInVersion=" + originalInterestedIn.knownForModelVersion,
            "originalInterestedIn=" + originalInterestedIn.clusterIdToScores.keySet,
            "filteredInterestedIn=" + filtered.knownForModelVersion,
            "filteredInterestedIn=" + filtered.clusterIdToScores.keySet,
            "optoutEntities=" + optoutEntities
          ).mkString("\t")
      }

    Util.printCounters(
      output.writeExecution(TypedTsv(outputDir))
    )
  }
}
