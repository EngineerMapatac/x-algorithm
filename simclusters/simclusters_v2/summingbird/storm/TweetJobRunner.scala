package com.twitter.simclusters_v2.summingbird.storm

import com.twitter.conversions.DurationOps._
import com.twitter.finagle.stats.NullStatsReceiver
import com.twitter.finagle.thrift.ClientId
import com.twitter.gizmoduck.thriftscala.User
import com.twitter.heron.util.CommonMetric
import com.twitter.scalding.Args
import com.twitter.simclusters_v2.summingbird.common.ClientConfigs
import com.twitter.simclusters_v2.summingbird.common.SimClustersProfile
import com.twitter.simclusters_v2.summingbird.common.SimClustersProfile.AltSetting
import com.twitter.simclusters_v2.summingbird.common.SimClustersProfile.Environment
import com.twitter.simclusters_v2.summingbird.stores.EntityClusterScoreReadableStore
import com.twitter.simclusters_v2.summingbird.stores.GizmoduckUserStore
import com.twitter.simclusters_v2.summingbird.stores.TopKClustersForTweetReadableStore
import com.twitter.simclusters_v2.summingbird.stores.TopKHydratedTweetsForClusterReadableStore
import com.twitter.simclusters_v2.summingbird.stores.TopKTweetsForClusterReadableStore
import com.twitter.simclusters_v2.summingbird.stores.TweetMetadata
import com.twitter.simclusters_v2.summingbird.stores.TweetMetadataStore
import com.twitter.simclusters_v2.summingbird.stores.TweetSquaredL2NormWritableStore
import com.twitter.simclusters_v2.summingbird.stores.UserInterestedInReadableStore
import com.twitter.simclusters_v2.thriftscala._
import com.twitter.storage.client.manhattan.kv.ManhattanKVClientMtlsParams
import com.twitter.summingbird.Options
import com.twitter.summingbird.TailProducer
import com.twitter.summingbird.online.option._
import com.twitter.summingbird.option._
import com.twitter.summingbird.storm.Storm
import com.twitter.summingbird.storm.StormMetric
import com.twitter.summingbird.storm.option.FlatMapStormMetrics
import com.twitter.summingbird.storm.option.SummerStormMetrics
import com.twitter.summingbird_internal.runner.common.JobName
import com.twitter.summingbird_internal.runner.common.SBRunConfig
import com.twitter.summingbird_internal.runner.storm.GenericRunner
import com.twitter.summingbird_internal.runner.storm.StormConfig
import com.twitter.summingbird_internal.sources.AppId
import com.twitter.unified_user_actions.client.config.KafkaConfigs
import com.twitter.unified_user_actions.client.summingbird.UnifiedUserActionsSourceScrooge

import java.lang
import org.apache.heron.api.{Config => HeronConfig}
import org.apache.heron.common.basics.ByteAmount
import org.apache.storm.{Config => BTConfig}

import scala.collection.JavaConverters._

object TweetJobRunner {
  def main(args: Array[String]): Unit = {
    GenericRunner(args, TweetStormJob(_))
  }
}

object TweetStormJob {

  import com.twitter.simclusters_v2.summingbird.common.Implicits._

  def jLong(num: Long): lang.Long = java.lang.Long.valueOf(num)
  def jInt(num: Int): Integer = java.lang.Integer.valueOf(num)
  def apply(args: Args): StormConfig = {

    lazy val env: String = args.getOrElse("env", "prod")
    lazy val zone: String = args.getOrElse("dc", "atla")
    lazy val clientId: ClientId = ClientId(s"simclusters_v2_summingbird.$env")

    lazy val profile = SimClustersProfile.fetchTweetJobProfile(Environment(env), AltSetting.Alt)

    lazy val stratoClient = ClientConfigs.stratoClient(profile.serviceIdentifier(zone))

    lazy val uuaEventSource =
      UnifiedUserActionsSourceScrooge(
        appId = AppId(profile.uuaSourceSubscribeId),
        parallelism = 150,
        kafkaConfig = KafkaConfigs.ProdUnifiedUserActionsEngagementOnly,
        skipToLatest = true
      ).source

    lazy val commonMetric =
      StormMetric(new CommonMetric(), CommonMetric.NAME, CommonMetric.POLL_INTERVAL)
    lazy val flatMapMetrics = FlatMapStormMetrics(Iterable(commonMetric))
    lazy val summerMetrics = SummerStormMetrics(Iterable(commonMetric))

    lazy val entityClusterScoreStore: Storm#Store[
      (SimClusterEntity, FullClusterIdBucket),
      ClustersWithScores
    ] = {
      Storm.store(
        EntityClusterScoreReadableStore
          .onlineMergeableStore(profile.entityClusterScorePath, profile.serviceIdentifier(zone)))
    }

    lazy val tweetTopKStore: Storm#Store[EntityWithVersion, TopKClustersWithScores] = {
      Storm.store(
        TopKClustersForTweetReadableStore
          .onlineMergeableStore(profile.tweetTopKClustersPath, profile.serviceIdentifier(zone)))
    }

    lazy val clusterTopKTweetsStore: Storm#Store[FullClusterId, TopKTweetsWithScores] = {
      Storm.store(
        TopKTweetsForClusterReadableStore
          .onlineMergeableStore(profile.clusterTopKTweetsPath, profile.serviceIdentifier(zone)))
    }

    lazy val clusterTopKHydratedTweetsStore: Storm#Store[
      FullClusterId,
      TopKHydratedTweetsWithScores
    ] = {
      Storm.store(
        TopKHydratedTweetsForClusterReadableStore
          .onlineMergeableStore(profile.clusterTopKTweetsPath, profile.serviceIdentifier(zone)))
    }

    lazy val clusterTopKVideoTweetsStore: Storm#Store[FullClusterId, TopKTweetsWithScores] = {
      Storm.store(
        TopKTweetsForClusterReadableStore
          .onlineMergeableStore(
            ClientConfigs.simClustersCoreAltVideoCachePath,
            profile.serviceIdentifier(zone)))
    }

    lazy val userInterestedInService: Storm#Service[Long, ClustersUserIsInterestedIn] = {
      Storm.service(
        UserInterestedInReadableStore.defaultStoreWithMtls(
          ManhattanKVClientMtlsParams(profile.serviceIdentifier(zone)),
          modelVersion = profile.modelVersionStr
        ))
    }

    lazy val tweetMetadataService: Storm#Service[SimClusterEntity, TweetMetadata] =
      Storm.service(
        TweetMetadataStore
          .tweetMetadataStore(stratoClient, "tweetypie/federated/summingbirdMigration.Tweet")
      )

    lazy val gizmoduckService: Storm#Service[Long, User] =
      Storm.service(
        GizmoduckUserStore
          .gizmoduckReadableStore(profile.serviceIdentifier(zone), clientId, NullStatsReceiver)
      )

    lazy val tweetSquaredL2NormStore = TweetSquaredL2NormWritableStore(
      profile.entityClusterScorePath,
      profile.serviceIdentifier(zone)
    )

    lazy val squaredL2NormSink: Storm#Sink[(EntityWithVersion, SquaredL2Norm)] = {
      Storm.sinkIntoWritable(tweetSquaredL2NormStore)
    }

    new StormConfig {

      val jobName: JobName = JobName(profile.jobName)

      implicit val jobID: JobId = JobId(jobName.toString)

      override def registrars =
        List(
          SBRunConfig.register[SimClusterEntity],
          SBRunConfig.register[FullClusterIdBucket],
          SBRunConfig.register[ClustersWithScores],
          SBRunConfig.register[EntityWithVersion],
          SBRunConfig.register[FullClusterId],
          SBRunConfig.register[EntityWithVersion],
          SBRunConfig.register[TopKEntitiesWithScores],
          SBRunConfig.register[TopKClustersWithScores],
          SBRunConfig.register[TopKTweetsWithScores],
          SBRunConfig.register[TopKHydratedTweetsWithScores]
        )

      override def vmSettings: Seq[String] = Seq()

      private val FlatMapPerWorker = 3
      private val SummerPerWorker = 3

      private val TotalWorker = 200

      override def transformConfig(config: Map[String, AnyRef]): Map[String, AnyRef] = {
        val heronConfig = new HeronConfig()

        val sourceName = "Tail.3-FlatMap-FlatMap-Summer-FlatMap-Source"
        val flatMapFlatMapSummerFlatMapName = "Tail.3-FlatMap-FlatMap-Summer-FlatMap"

        val tailFlatMapName = "Tail-FlatMap"

        val TotalRamRB = 216
        val HeavyMemGB = 12

        heronConfig.setComponentRam(sourceName, ByteAmount.fromGigabytes(HeavyMemGB))
        heronConfig.setComponentRam(
          flatMapFlatMapSummerFlatMapName,
          ByteAmount.fromGigabytes(HeavyMemGB))
        heronConfig.setComponentRam(tailFlatMapName, ByteAmount.fromGigabytes(HeavyMemGB))
        heronConfig.setContainerRamRequested(ByteAmount.fromGigabytes(TotalRamRB))

        val TotalCPU = jLong(36)
        val HeavyCPU = 4
        heronConfig.setContainerCpuRequested(TotalCPU.toDouble)
        heronConfig.setComponentCpu(tailFlatMapName, HeavyCPU.toDouble)

        super.transformConfig(config) ++ List(
          BTConfig.TOPOLOGY_TEAM_NAME -> "cassowary",
          BTConfig.TOPOLOGY_TEAM_EMAIL -> "recos-platform-staff@twitter.com",
          BTConfig.TOPOLOGY_WORKERS -> jInt(TotalWorker),
          BTConfig.TOPOLOGY_ACKER_EXECUTORS -> jInt(0),
          BTConfig.TOPOLOGY_MESSAGE_TIMEOUT_SECS -> jInt(45),
          BTConfig.TOPOLOGY_WORKER_CHILDOPTS -> List(
            "-XX:MaxMetaspaceSize=256M",
            "-Djava.security.auth.login.config=config/jaas.conf",
            "-Dsun.security.krb5.debug=true",
            "-Dcom.twitter.eventbus.client.EnableKafkaSaslTls=true",
            "-Dcom.twitter.eventbus.client.zoneName=" + zone
          ).mkString(" "),
          "storm.job.uniqueId" -> jobID.get
        ) ++ heronConfig.asScala.toMap
      }

      override def getNamedOptions: Map[String, Options] = Map(
        "DEFAULT" -> Options()
          .set(FlatMapParallelism(TotalWorker * FlatMapPerWorker))
          .set(SourceParallelism(TotalWorker))
          .set(SummerBatchMultiplier(1000))
          .set(CacheSize(10000))
          .set(flatMapMetrics)
          .set(summerMetrics),
        TweetJob.NodeName.TweetClusterUpdatedScoresFlatMapNodeName -> Options()
          .set(FlatMapParallelism(TotalWorker * FlatMapPerWorker)),
        TweetJob.NodeName.TweetClusterScoreSummerNodeName -> Options()
          .set(SummerParallelism(TotalWorker * SummerPerWorker * 4))
          .set(FlushFrequency(60.seconds)),
        TweetJob.NodeName.ClusterTopKTweetsNodeName -> Options()
          .set(SummerParallelism(TotalWorker * SummerPerWorker))
          .set(FlushFrequency(30.seconds)),
        TweetJob.NodeName.ClusterTopKVideoTweetsNodeName -> Options()
          .set(SummerParallelism(TotalWorker * SummerPerWorker))
          .set(FlushFrequency(30.seconds)),
        TweetJob.NodeName.ClusterTopKHydratedTweetsNodeName -> Options()
          .set(SummerParallelism(TotalWorker * SummerPerWorker))
          .set(FlushFrequency(30.seconds)),
        TweetJob.NodeName.TweetTopKNodeName -> Options()
          .set(SummerParallelism(TotalWorker * SummerPerWorker))
          .set(FlushFrequency(30.seconds))
      )

      override def graph: TailProducer[Storm, Any] = TweetJob.generate[Storm](
        profile,
        uuaEventSource,
        userInterestedInService,
        entityClusterScoreStore,
        tweetTopKStore,
        clusterTopKTweetsStore,
        clusterTopKHydratedTweetsStore,
        clusterTopKVideoTweetsStore,
        tweetMetadataService,
        squaredL2NormSink
      )
    }
  }
}
