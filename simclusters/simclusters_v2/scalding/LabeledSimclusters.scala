package com.twitter.simclusters_v2.scalding

import com.twitter.dal.client.dataset.KeyValDALDataset
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.scalding_internal.job.analytics_batch._
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.simclusters_v2.common.ModelVersions
import com.twitter.simclusters_v2.hdfs_sources.AdhocKeyValSources
import com.twitter.simclusters_v2.hdfs_sources.DataPaths
import com.twitter.simclusters_v2.hdfs_sources.LabeledSimclustersV2InterestedIn20M145K2020ScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.SimclustersV2InterestedIn20M145K2020ScalaDataset
import com.twitter.simclusters_v2.thriftscala._

object TypedTsvWithHeader extends TypedSeperatedFile {
  val separator = "\t"
  override def skipHeader: Boolean = true
}

object LabeledSimclustersInterestedIn extends LabeledSimclustersBatchBase {
  override val firstTime: String = "2023-12-16"
  override val outputKVDataset: KeyValDALDataset[KeyVal[Long, LabeledClustersPerUser]] =
    LabeledSimclustersV2InterestedIn20M145K2020ScalaDataset
  override val outputPath: String = DataPaths.LabeledInterestedIn2020Path
}

trait LabeledSimclustersBatchBase extends TwitterScheduledExecutionApp {
  implicit val tz = DateOps.UTC
  implicit val parser = DateParser.default

  def firstTime: String
  val batchIncrement: Duration = Days(2)

  def outputKVDataset: KeyValDALDataset[KeyVal[Long, LabeledClustersPerUser]]
  def outputPath: String

  private lazy val execArgs = AnalyticsBatchExecutionArgs(
    batchDesc = BatchDescription(this.getClass.getName.replace("$", "")),
    firstTime = BatchFirstTime(RichDate(firstTime)),
    lastTime = None,
    batchIncrement = BatchIncrement(batchIncrement)
  )

  override def scheduledJob: Execution[Unit] = AnalyticsBatchExecution(execArgs) {
    implicit dateRange =>
      Execution.withId { implicit uniqueId =>
        Execution.withArgs { args =>
          val interestedInSimclusters: TypedPipe[KeyVal[Long, ClustersUserIsInterestedIn]] =
            DAL
              .readMostRecentSnapshot(
                SimclustersV2InterestedIn20M145K2020ScalaDataset,
              ).toTypedPipe

          val result = LabeledSimclusters.run(interestedInSimclusters)

          val writeKeyValResultExec = result
            .map { case (userId, labeledClusters) => KeyVal(userId, labeledClusters) }
            .writeDALVersionedKeyValExecution(
              outputKVDataset,
              D.Suffix(outputPath)
            )

          writeKeyValResultExec
        }
      }
  }
}

object LabeledSimclusters {

  type ClusterNames = String
  type ClusterId = Int
  type UserId = Long
  def run(
    interestedInSimclusters: TypedPipe[KeyVal[UserId, ClustersUserIsInterestedIn]]
  ): TypedPipe[(UserId, LabeledClustersPerUser)] = {
    lazy val clusterIdToLabelName =
      TypedPipe
        .from(
          TypedTsvWithHeader[(ClusterId, ClusterNames)](
            "/atla/proc/user/cassowary/adhoc/simcluster_labels/simcluster_labels.tsv")).collect {
          case (clusterId, names) if names != null =>
            clusterId -> names.split('|').toSeq
        }

    val clusterIdToUsers =
      interestedInSimclusters.collect {
        case KeyVal(
              userId,
              ClustersUserIsInterestedIn(ModelVersions.Model20M145K2020, clusterIdToScores)) =>
          clusterIdToScores.map {
            case (clusterId, userToInterestedInClusterScores: UserToInterestedInClusterScores) =>
              (
                clusterId,
                (
                  userId,
                  userToInterestedInClusterScores.followScoreClusterNormalizedOnly,
                  userToInterestedInClusterScores.favScoreClusterNormalizedOnly))
          }
      }.flatten

    clusterIdToUsers
      .hashJoin(clusterIdToLabelName).groupBy {
        case (_, ((userId, _, _), _)) => userId
      }
      .toList.map {
        case (userId, values) =>
          val details = values.map {
            case (clusterId, ((_, followScoreOpt, favScoreOpt), clusterNames)) =>
              clusterId -> LabelMetadata(Some(clusterNames), followScoreOpt, favScoreOpt)
          }.toMap
          userId -> LabeledClustersPerUser(details)
      }
  }
}

object LabeledSimclustersAdhoc extends TwitterExecutionApp {
  implicit val tz = DateOps.UTC
  implicit val parser = DateParser.default
  def job: Execution[Unit] =
    Execution.getConfigMode.flatMap {
      case (config, mode) =>
        Execution.withId { implicit uniqueId =>
          val args = config.getArgs
          val outputSink = AdhocKeyValSources.labeledInterestedInSource(args("outputDir"))
          implicit val date: DateRange = DateRange.parse(args("date"))

          val interestedInSimclusters: TypedPipe[KeyVal[Long, ClustersUserIsInterestedIn]] =
            DAL
              .readMostRecentSnapshot(
                SimclustersV2InterestedIn20M145K2020ScalaDataset,
              ).toTypedPipe

          LabeledSimclusters
            .run(
              interestedInSimclusters
            ).writeExecution(outputSink)
        }
    }
}
