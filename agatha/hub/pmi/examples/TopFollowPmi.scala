package com.twitter.hub.pmi.examples

import com.twitter.bijection.Injection
import com.twitter.hub.pmi.PMI._
import com.twitter.hub.pmi.PMITypes.SmallToSmallPMIContainer
import com.twitter.hub.pmi.PMITypes.SymmetricPMIContainer
import com.twitter.hub.pmi.math.MathParams
import com.twitter.scalding._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.AllowCrossClusterSameDC
import com.twitter.scalding_internal.job.RequiredBinaryComparators.ordSer
import com.twitter.scalding_internal.job.RequiredBinaryComparatorsExecutionApp
import com.twitter.wtf.scalding.jobs.common.AdhocExecutionApp
import graphstore.common.FlockFollowsJavaDataset
import java.util.TimeZone
import com.twitter.usersource.snapshot.combined.UsersourceScalaDataset

object TopFollowPmi extends RequiredBinaryComparatorsExecutionApp with AdhocExecutionApp {
  def runOnDateRange(
    args: Args
  )(
    implicit dateRange: DateRange,
    timeZone: TimeZone,
    uniqueID: UniqueID
  ): Execution[Unit] = {
    val followGraph = DataUtils.getMedPlusFollowGraph.forceToDisk
    val topMillionProducers = DataUtils.getTopNProducers(followGraph)
    val allConsumerTopProducerGraph = DataUtils
      .filterGraphToTopNProducers(
        followGraph,
        topMillionProducers,
      ).forceToDisk

    import BigPMIConfig._

    bigPMI(
      SymmetricPMIContainer(
        allConsumerTopProducerGraph.map { case (consumer, producer) => (consumer, producer, 1.0) },
      ),
      optFeatureCrossReducers = Some(10000),
    ).map(pmiStats => (pmiStats.featureId1, pmiStats.featureId2, pmiStats.pmi))
      .writeExecution(TypedTsv("/gcs/user/mst/adhoc/topMillionProducerPmiMatrixSymmetric"))
  }
}

object DataUtils {
  type User = Long
  type EdgeLite = (User, User)

  def getMedPlusUsers(
    implicit dateRange: DateRange,
  ): TypedPipe[Long] = {
    DAL
      .readMostRecentSnapshot(UsersourceScalaDataset, dateRange)
      .withRemoteReadPolicy(AllowCrossClusterSameDC)
      .toTypedPipe
      .flatMap { combinedUser =>
        for {
          user <- combinedUser.user
          safety <- user.safety
          if !(safety.deactivated || safety.suspended)
          extendedUser <- combinedUser.userExtended
          userState <- extendedUser.userState
          if userState.value >= 4
        } yield user.id
      }
  }

  def getFollowGraph(implicit dateRange: DateRange): TypedPipe[EdgeLite] = {
    DAL
      .readMostRecentSnapshot(FlockFollowsJavaDataset, dateRange)
      .toTypedPipe
      .filter(edge => edge.getStateId == 0)
      .map(edge => (edge.getSourceId, edge.getDestinationId))
  }

  def getMedPlusFollowGraph(implicit dateRange: DateRange): TypedPipe[EdgeLite] = {
    getFollowGraph
      .groupBy(_._1)
      .join(getMedPlusUsers.asKeys)
      .mapValues(_._1)
      .values
  }

  def getTopNProducers(
    graph: TypedPipe[EdgeLite],
    N: Int = 1000000
  ): TypedPipe[User] = {
    graph
      .groupBy(_._2)
      .size
      .groupAll
      .sortedReverseTake(N)(Ordering.by(_._2))
      .flattenValues
      .mapValues(_._1)
      .values
  }

  def filterGraphToTopNProducers(
    graph: TypedPipe[EdgeLite],
    topNProducers: TypedPipe[User]
  ): TypedPipe[EdgeLite] = {
    graph
      .groupBy(_._2)
      .join(topNProducers.asKeys)
      .mapValues(_._1)
      .values
  }
}

object BigPMIConfig {
  implicit val pmiParams: RegularizationParams = RegularizationParams(
    None,
    Some(175),
    None,
  )

  implicit val mathParams: MathParams = MathParams(
    pseudoCounts = 0,
    z = 2.0,
    positive = true,
    pmiDiscount = math.log(5),
    minAbsPMI = 0
  )

  implicit val serializeUserId: Long => Array[Byte] =
    implicitly[Injection[Long, Array[Byte]]].toFunction
}
