package com.twitter.simclusters_v2.scalding.offline_data_conversion

import com.twitter.finagle.mtls.authentication.ServiceIdentifier
import com.twitter.stitch.Stitch
import com.twitter.strato.catalog.Fetch
import com.twitter.strato.client.Client
import com.twitter.strato.client.Strato
import com.twitter.strato.generated.client.recommendations.simclusters_v2.SimclusterIdToSnowflakeIdClientColumn
import com.twitter.strato.generated.client.recommendations.simclusters_v2.SnowflakeIdToSimclusterLabelClientColumn
import com.twitter.util.Future

object HydrationHelper {
  com.twitter.server.Init()
  @transient private lazy val client: Client = Strato.client
    .withMutualTls(ServiceIdentifier("cassowary", "offline_data_conversion", "devel", "atla"))
    .build()
  private val clusterIdToSnowflake = new SimclusterIdToSnowflakeIdClientColumn(client)
  private val clusterIdToSnowflakeIdFetcher = clusterIdToSnowflake.fetcher

  private val snowflakeToClusterLabel = new SnowflakeIdToSimclusterLabelClientColumn(client)
  private val snowflakeIdToClusterLabelFetcher = snowflakeToClusterLabel.fetcher

  def hydrateClusterIdToLabelName(id: Int): Future[Fetch.Result[String]] = {

    Stitch.run(
      clusterIdToSnowflakeIdFetcher.fetch(id).flatMap { result =>
        result.v match {
          case Some(snowflakeId) =>
            snowflakeIdToClusterLabelFetcher.fetch(snowflakeId)
          case _ => Stitch.value(Fetch.Result(None))
        }
      }
    )
  }
}
