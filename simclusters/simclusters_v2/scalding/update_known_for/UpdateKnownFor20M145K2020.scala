package com.twitter.simclusters_v2.scalding.update_known_for

import com.twitter.bijection.scrooge.BinaryScalaCodec
import com.twitter.hermit.candidate.thriftscala.Candidates
import com.twitter.logging.Logger
import com.twitter.pluck.source.cassowary.FollowingsCosineSimilaritiesManhattanSource
import com.twitter.scalding.typed.TypedPipe
import com.twitter.scalding.DateOps
import com.twitter.scalding.DateParser
import com.twitter.scalding.Days
import com.twitter.scalding.Execution
import com.twitter.scalding.RichDate
import com.twitter.scalding.TypedTsv
import com.twitter.scalding.UniqueID
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite.D
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.dalv2.remote_access.AllowCrossClusterSameDC
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.simclusters_v2.common.ClusterId
import com.twitter.simclusters_v2.common.ModelVersions
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.hdfs_sources.AdhocKeyValSources
import com.twitter.simclusters_v2.hdfs_sources.InternalDataPaths
import com.twitter.simclusters_v2.hdfs_sources.SimclustersV2KnownFor20M145KDec11ScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.SimclustersV2KnownFor20M145KUpdatedScalaDataset
import com.twitter.simclusters_v2.hdfs_sources.SimclustersV2RawKnownFor20M145K2020ScalaDataset
import com.twitter.simclusters_v2.scalding.KnownForSources
import com.twitter.simclusters_v2.scalding.KnownForSources.fromKeyVal
import com.twitter.simclusters_v2.scalding.common.Util
import com.twitter.wtf.scalding.jobs.common.ScheduledExecutionApp
import java.util.TimeZone

object UpdateKnownFor20M145K2020 extends ScheduledExecutionApp {

  override val firstTime: RichDate = RichDate("2020-10-04")

  override val batchIncrement: Duration = Days(7)

  private val tempLocationPath = "/user/cassowary/temp/simclusters_v2/known_for_20m_145k_2020"

  private val simsGraphPath =
    "/atla/proc/user/cassowary/manhattan_sequence_files/approximate_cosine_similarity_follow"

  override def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {

    Execution.getConfigMode.flatMap {
      case (_, mode) =>
        implicit def valueCodec: BinaryScalaCodec[Candidates] = BinaryScalaCodec(Candidates)
        val minActiveFollowers = args.int("minActiveFollowers", 400)
        val topK = args.int("topK", 20000000)

        val maxNeighbors = args.int("maxNeighbors", 400)

        val squareWeightsEnable = args.boolean("squareWeightsEnable")

        val maxEpochsForClustering = args.int("maxEpochs", 3)
        val wtCoeff = args.double("wtCoeff", 10.0)

        val previousKnownFor: TypedPipe[(UserId, Array[(ClusterId, Float)])] =
          fromKeyVal(
            DAL
              .readMostRecentSnapshot(
                SimclustersV2RawKnownFor20M145K2020ScalaDataset,
                dateRange.embiggen(Days(30)))
              .withRemoteReadPolicy(AllowCrossClusterSameDC)
              .toTypedPipe,
            ModelVersions.Model20M145K2020
          )

        UpdateKnownForSBFRunner
          .runUpdateKnownFor(
            TypedPipe
              .from(FollowingsCosineSimilaritiesManhattanSource(simsGraphPath))
              .map(_._2),
            minActiveFollowers,
            topK,
            maxNeighbors,
            tempLocationPath,
            previousKnownFor,
            maxEpochsForClustering,
            squareWeightsEnable,
            wtCoeff,
            mode
          )
          .flatMap { updateKnownFor =>
            Execution
              .zip(
                KnownForSources
                  .toKeyVal(updateKnownFor, ModelVersions.Model20M145K2020)
                  .writeDALVersionedKeyValExecution(
                    SimclustersV2RawKnownFor20M145K2020ScalaDataset,
                    D.Suffix(InternalDataPaths.RawKnownFor2020Path)
                  ),
                UpdateKnownForSBFRunner
                  .evaluateUpdatedKnownFor(updateKnownFor, previousKnownFor)
                  .flatMap { emailText =>
                    Util
                      .sendEmail(
                        emailText,
                        s"Change in cluster assignments for new KnownFor ModelVersion: 20M145K2020",
                        "simclusters-v2-alerts@twitter.com")
                    Execution.unit
                  }
              ).unit
          }
    }
  }
}

object UpdateKnownFor20M145K2020Adhoc extends TwitterExecutionApp {
  implicit val tz: java.util.TimeZone = DateOps.UTC
  implicit val dp = DateParser.default
  val log = Logger()

  def job: Execution[Unit] =
    Execution.getConfigMode.flatMap {
      case (config, mode) =>
        Execution.withId { implicit uniqueId =>
          val args = config.getArgs

          implicit def valueCodec: BinaryScalaCodec[Candidates] = BinaryScalaCodec(Candidates)
          val minActiveFollowers = args.int("minActiveFollowers", 400)
          val topK = args.int("topK", 20000000)

          val clusterAssignmentOutput = args("outputClusterDir")
          val maxNeighbors = args.int("maxNeighbors", 400)

          val squareWeightsEnable = args.boolean("squareWeightsEnable")

          val maxEpochsForClustering = args.int("maxEpochs", 3)
          val wtCoeff = args.double("wtCoeff", 10.0)

          val simsGraphPath =
            "/atla/proc/user/cassowary/manhattan_sequence_files/approximate_cosine_similarity_follow"
          val inputPreviousKnownFor: TypedPipe[(Long, Array[(Int, Float)])] =
            args.optional("inputPreviousKnownForDataSet") match {
              case Some(inputKnownForDir) =>
                println(
                  "Input knownFors provided, using these as the initial cluster assignments for users")
                TypedPipe
                  .from(AdhocKeyValSources.knownForSBFResultsDevelSource(inputKnownForDir))
              case None =>
                println(
                  "Using knownFor Assignments from prod as no previous assignment was provided in the input")
                if (args.boolean("dec11")) {
                  KnownForSources
                    .fromKeyVal(
                      DAL
                        .readMostRecentSnapshotNoOlderThan(
                          SimclustersV2KnownFor20M145KDec11ScalaDataset,
                          Days(30)).withRemoteReadPolicy(AllowCrossClusterSameDC).toTypedPipe,
                      ModelVersions.Model20M145KDec11
                    )
                } else {
                  KnownForSources
                    .fromKeyVal(
                      DAL
                        .readMostRecentSnapshotNoOlderThan(
                          SimclustersV2KnownFor20M145KUpdatedScalaDataset,
                          Days(30)).withRemoteReadPolicy(AllowCrossClusterSameDC).toTypedPipe,
                      ModelVersions.Model20M145KUpdated
                    )
                }
            }
          UpdateKnownForSBFRunner
            .runUpdateKnownFor(
              TypedPipe
                .from(FollowingsCosineSimilaritiesManhattanSource(simsGraphPath))
                .map(_._2),
              minActiveFollowers,
              topK,
              maxNeighbors,
              clusterAssignmentOutput,
              inputPreviousKnownFor,
              maxEpochsForClustering,
              squareWeightsEnable,
              wtCoeff,
              mode
            )
            .flatMap { updateKnownFor =>
              Execution
                .zip(
                  updateKnownFor
                    .mapValues(_.toList).writeExecution(TypedTsv(clusterAssignmentOutput)),
                  updateKnownFor.writeExecution(AdhocKeyValSources.knownForSBFResultsDevelSource(
                    clusterAssignmentOutput + "_KeyVal")),
                  UpdateKnownForSBFRunner
                    .evaluateUpdatedKnownFor(updateKnownFor, inputPreviousKnownFor)
                    .flatMap { emailText =>
                      Util
                        .sendEmail(
                          emailText,
                          s"Change in cluster assignments for new KnownFor ModelVersion: 20M145K2020" + clusterAssignmentOutput,
                          "simclusters-v2-alerts@twitter.com")
                      Execution.unit
                    }
                ).unit
            }
        }
    }
}
