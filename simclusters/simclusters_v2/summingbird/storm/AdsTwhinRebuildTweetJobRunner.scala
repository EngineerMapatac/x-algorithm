package com.twitter.simclusters_v2.summingbird.storm

import com.twitter.conversions.DurationOps._
import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.heron.util.CommonMetric
import com.twitter.scalding.Args
import com.twitter.simclusters_v2.common.TweetId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.common.VersionId
import com.twitter.simclusters_v2.summingbird.common.ClientConfigs
import com.twitter.simclusters_v2.summingbird.common.Configs
import com.twitter.simclusters_v2.summingbird.stores.AdsTwhinTweetEmbeddingStore
import com.twitter.simclusters_v2.summingbird.stores.AdsTwhinUserEmbeddingStore
import com.twitter.simclusters_v2.thriftscala._
import com.twitter.storehaus.FutureCollector
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

object AdsTwhinRebuildTweetJobRunner {
  def main(args: Array[String]): Unit = {
    GenericRunner(args, AdsTwhinRebuildTweetStormJob(_))
  }
}

object AdsTwhinRebuildTweetStormJob {

  import com.twitter.simclusters_v2.summingbird.common.Implicits._

  def jLong(num: Long): lang.Long = java.lang.Long.valueOf(num)
  def jInt(num: Int): Integer = java.lang.Integer.valueOf(num)
  def apply(args: Args): StormConfig = {

    lazy val env: String = args.getOrElse("env", "prod")
    lazy val zone: String = args.getOrElse("dc", "atla")

    lazy val serviceIdentifier =
      ServiceIdentifier(Configs.adsRole, "summingbird_ads_twhin_rebuild_tweet_job", env, zone)

    lazy val stratoClient = ClientConfigs.stratoClient(serviceIdentifier)

    lazy val uuaEventSource =
      UnifiedUserActionsSourceScrooge(
        appId = AppId("ads_twhin_rebuild_tweet_uua_events_prod"),
        parallelism = 150,
        kafkaConfig = KafkaConfigs.ProdUnifiedUserActionsEngagementOnly
      ).source

    lazy val commonMetric =
      StormMetric(new CommonMetric(), CommonMetric.NAME, CommonMetric.POLL_INTERVAL)
    lazy val flatMapMetrics = FlatMapStormMetrics(Iterable(commonMetric))
    lazy val summerMetrics = SummerStormMetrics(Iterable(commonMetric))

    lazy val tweetEmbeddingStore: Storm#Store[
      (TweetId, VersionId),
      PersistentTwhinTweetEmbedding
    ] = {

      import com.twitter.storehaus.algebra.StoreAlgebra._

      lazy val mergeableStore = AdsTwhinTweetEmbeddingStore
        .persistentVersionedTweetEmbeddingStore(
          stratoClient,
          AdsTwhinTweetEmbeddingStore.prodPersistentRebuildTweetStratoColumn
        ).toMergeable(
          mon = persistentTwhinTweetEmbeddingMonoid,
          fc = implicitly[FutureCollector]
        )
      Storm.onlineOnlyStore(mergeableStore)
    }

    lazy val userEmbeddingService: Storm#Service[UserId, TwhinTweetEmbedding] = {
      Storm.service(AdsTwhinUserEmbeddingStore
        .userRebuildEmbeddingStore(stratoClient, AdsTwhinUserEmbeddingStore.prodRebuildUserColumn))
    }

    lazy val refreshedUserEmbeddingService: Storm#Service[UserId, TwhinTweetEmbedding] = {
      Storm.service(
        AdsTwhinUserEmbeddingStore
          .userRebuildEmbeddingStore(stratoClient, AdsTwhinUserEmbeddingStore.refreshedUserColumn)
      )
    }

    new StormConfig {

      val jobName: JobName = JobName("ads_twhin_rebuild_tweet_job")

      implicit val jobID: JobId = JobId(jobName.toString)

      override def registrars =
        List(
          SBRunConfig.register[TwhinTweetEmbedding],
          SBRunConfig.register[PersistentTwhinTweetEmbedding],
          SBRunConfig.register[KeyedPersistentTwhinTweetEmbedding],
        )

      override def vmSettings: Seq[String] = Seq()

      private val FlatMapPerWorker = 3
      private val SummerPerWorker = 3

      private val TotalWorker = 150

      override def transformConfig(config: Map[String, AnyRef]): Map[String, AnyRef] = {
        val heronConfig = new HeronConfig()

        val TotalRamRB = 160
        val HeavyMemGB = 10

        heronConfig.setContainerRamRequested(ByteAmount.fromGigabytes(TotalRamRB))

        val TotalCPU = jLong(48)
        heronConfig.setContainerCpuRequested(TotalCPU.toDouble)

        super.transformConfig(config) ++ List(
          BTConfig.TOPOLOGY_TEAM_NAME -> "ads-prediction",
          BTConfig.TOPOLOGY_TEAM_EMAIL -> "ads-usermodeling@twitter.com",
          BTConfig.TOPOLOGY_WORKERS -> jInt(TotalWorker),
          BTConfig.TOPOLOGY_ACKER_EXECUTORS -> jInt(0),
          BTConfig.TOPOLOGY_MESSAGE_TIMEOUT_SECS -> jInt(30),
          BTConfig.TOPOLOGY_WORKER_CHILDOPTS -> List(
            "-XX:MaxMetaspaceSize=256M",
            "-Djava.security.auth.login.config=config/ads-prediction-jaas.conf",
            "-Dsun.security.krb5.debug=true",
            "-Dcom.twitter.eventbus.client.EnableKafkaSaslTls=true",
            "-Dcom.twitter.eventbus.client.zoneName=" + zone
          ).mkString(" "),
          "storm.job.uniqueId" -> jobID.get
        ) ++ heronConfig.asScala.toMap
      }

      private val baseOptions = Options()
        .set(SourceParallelism(TotalWorker))
        .set(flatMapMetrics)
        .set(FlatMapParallelism(TotalWorker * FlatMapPerWorker))
        .set(summerMetrics)
        .set(SummerBatchMultiplier(1000))
        .set(SummerParallelism(TotalWorker * SummerPerWorker))

      override def getNamedOptions: Map[String, Options] = Map(
        "DEFAULT" -> baseOptions,
        TwhinRebuildTweetJob.NodeName.TweetEmbeddingSummerNodeName -> Options()
          .set(CacheSize(10000))
          .set(FlushFrequency(60.seconds)),
        TwhinRebuildTweetJob.NodeName.RefreshedTweetEmbeddingSummerNodeName -> Options()
          .set(CacheSize(10000))
          .set(FlushFrequency(60.seconds)),
      )

      override def graph: TailProducer[Storm, Any] = AdsTwhinRebuildTweetJob.generate[Storm](
        uuaEventSource,
        userEmbeddingService,
        refreshedUserEmbeddingService,
        tweetEmbeddingStore
      )
    }
  }

}
