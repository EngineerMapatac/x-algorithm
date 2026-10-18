package com.twitter.simclusters_v2.scalding.offline_job

import com.twitter.scalding._
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.simclusters_v2.common.ModelVersions
import com.twitter.simclusters_v2.hdfs_sources.AdhocKeyValSources
import com.twitter.simclusters_v2.hdfs_sources.ClusterTopKTweetsHourlySuffixSource
import com.twitter.simclusters_v2.hdfs_sources.TweetClusterScoresHourlySuffixSource
import com.twitter.simclusters_v2.hdfs_sources.TweetTopKClustersHourlySuffixSource
import com.twitter.simclusters_v2.scalding.common.Util
import com.twitter.simclusters_v2.scalding.offline_job.SimClustersOfflineJob._
import com.twitter.simclusters_v2.thriftscala.ClustersUserIsInterestedIn
import java.util.TimeZone

object SimClustersOfflineJobAdhocApp extends TwitterExecutionApp {

  import SimClustersOfflineJobUtil._
  import com.twitter.simclusters_v2.scalding.common.TypedRichPipe._

  override def job: Execution[Unit] =
    Execution.withId { implicit uniqueId =>
      Execution.withArgs { args: Args =>
        val wholeDateRange: DateRange = DateRange.parse(args.list("date"))
        val batchSize: Duration = Hours(args.int("batch_hours"))

        val outputDir = args("output_dir")

        val modelVersion = args.getOrElse("model_version", ModelVersions.Model20M145K2020)

        val scoringMethod = args.getOrElse("score", "logFav")

        val tweetClusterScoreOutputPath: String = outputDir + "/tweet_cluster_scores"

        val tweetTopKClustersOutputPath: String = outputDir + "/tweet_top_k_clusters"

        val clusterTopKTweetsOutputPath: String = outputDir + "/cluster_top_k_tweets"

        val fullInterestedInData: TypedPipe[(Long, ClustersUserIsInterestedIn)] =
          args.optional("interested_in_path") match {
            case Some(dir) =>
              println("Loading InterestedIn from supplied path " + dir)
              TypedPipe.from(AdhocKeyValSources.interestedInSource(dir))
            case None =>
              println("Loading production InterestedIn data")
              readInterestedInScalaDataset(wholeDateRange)
          }

        val interestedInData: TypedPipe[(Long, ClustersUserIsInterestedIn)] =
          fullInterestedInData.filter(_._2.knownForModelVersion == modelVersion)

        val debugExec = Execution.zip(
          fullInterestedInData.printSummary("fullInterestedIn", numRecords = 20),
          interestedInData.printSummary("interestedIn", numRecords = 20)
        )

        def runBatch(batchDateRange: DateRange): Execution[Unit] = {
          if (batchDateRange.start.timestamp > wholeDateRange.end.timestamp) {
            Execution.unit
          } else {

            val previousScores = if (batchDateRange.start == wholeDateRange.start) {
              TypedPipe.from(Nil)
            } else {
              TypedPipe.from(
                TweetClusterScoresHourlySuffixSource(
                  tweetClusterScoreOutputPath,
                  batchDateRange - batchSize
                )
              )
            }

            val latestScores = computeAggregatedTweetClusterScores(
              batchDateRange,
              interestedInData,
              readTimelineFavoriteData(batchDateRange),
              previousScores
            )

            val writeLatestScoresExecution = {
              Execution.zip(
                latestScores.printSummary(name = "TweetEntityScores"),
                latestScores
                  .writeExecution(
                    TweetClusterScoresHourlySuffixSource(
                      tweetClusterScoreOutputPath,
                      batchDateRange
                    )
                  )
              )
            }

            val computeTweetTopKExecution = {
              val tweetTopK = computeTweetTopKClusters(latestScores)
              Execution.zip(
                tweetTopK.printSummary(name = "TweetTopK"),
                tweetTopK.writeExecution(
                  TweetTopKClustersHourlySuffixSource(tweetTopKClustersOutputPath, batchDateRange)
                )
              )
            }

            val computeClusterTopKExecution = {
              val clusterTopK = computeClusterTopKTweets(latestScores)
              Execution.zip(
                clusterTopK.printSummary(name = "ClusterTopK"),
                clusterTopK.writeExecution(
                  ClusterTopKTweetsHourlySuffixSource(clusterTopKTweetsOutputPath, batchDateRange)
                )
              )
            }

            Execution
              .zip(
                writeLatestScoresExecution,
                computeTweetTopKExecution,
                computeClusterTopKExecution
              ).flatMap { _ =>
                runBatch(batchDateRange + batchSize)
              }
          }
        }

        Util.printCounters(
          Execution.zip(
            debugExec,
            runBatch(
              DateRange(wholeDateRange.start, wholeDateRange.start + batchSize - Millisecs(1)))
          )
        )
      }
    }
}

object DumpClusterTopKTweetsAdhoc extends TwitterExecutionApp {

  implicit val timeZone: TimeZone = DateOps.UTC
  implicit val dateParser: DateParser = DateParser.default

  import com.twitter.simclusters_v2.scalding.common.TypedRichPipe._
  import com.twitter.simclusters_v2.summingbird.common.ThriftDecayedValueMonoid._

  override def job: Execution[Unit] =
    Execution.withId { implicit uniqueId =>
      Execution.withArgs { args: Args =>
        val date = DateRange.parse(args.list("date"))
        val path = args("clusterTopKTweetsPath")
        val input = TypedPipe.from(ClusterTopKTweetsHourlySuffixSource(path, date))
        val clusters = args.list("clusters").map(_.toInt).toSet

        val dvm = SimClustersOfflineJobUtil.thriftDecayedValueMonoid
        if (clusters.isEmpty) {
          input.printSummary("Cluster top k tweets")
        } else {
          input
            .collect {
              case rec if clusters.contains(rec.clusterId) =>
                val res = rec.topKTweets
                  .mapValues { x =>
                    x.score
                      .map { y =>
                        val enriched = new EnrichedThriftDecayedValue(y)(dvm)
                        enriched.decayToTimestamp(date.end.timestamp).value
                      }.getOrElse(0.0)
                  }.toList.sortBy(-_._2)
                rec.clusterId + "\t" + Util.prettyJsonMapper
                  .writeValueAsString(res).replaceAll("\n", " ")
            }
            .toIterableExecution
            .map { strings => println(strings.mkString("\n")) }
        }
      }
    }
}
