package com.twitter.simclusters_v2.scalding

import com.twitter.scalding.DateOps
import com.twitter.scalding.DateParser
import com.twitter.scalding.Execution
import com.twitter.scalding.Stat
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.TypedTsv
import com.twitter.scalding.UniqueID
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.simclusters_v2.common.ClusterId
import com.twitter.simclusters_v2.common.UserId
import com.twitter.simclusters_v2.scalding.common.Util
import com.twitter.simclusters_v2.scalding.common.Util.Distribution

object CompareClusters {
  def norm(a: Iterable[Float]): Float = {
    math
      .sqrt(a.map { x => x * x }.sum).toFloat
  }

  def cosine(a: Map[Long, Float], b: Map[Long, Float]): Float = {
    val intersect = a.toList.collect {
      case (id, score) if b.contains(id) =>
        score * b(id)
    }
    val dot = if (intersect.nonEmpty) intersect.sum else 0
    val aNorm = norm(a.values)
    val bNorm = norm(b.values)
    if (aNorm > 0 && bNorm > 0) {
      dot / aNorm / bNorm
    } else 0
  }

  def compareClusterAssignments(
    newKnownFor: TypedPipe[(UserId, List[(ClusterId, Float)])],
    oldKnownFor: TypedPipe[(UserId, List[(ClusterId, Float)])]
  )(
    implicit uniqueID: UniqueID
  ): Execution[String] = {

    val emptyToSomething = Stat("no_assignment_to_some")
    val somethingToEmpty = Stat("some_assignment_to_none")
    val emptyToEmpty = Stat("empty_to_empty")
    val sameCluster = Stat("same_cluster")
    val diffCluster = Stat("diff_cluster")

    val calculateStatExec = newKnownFor
      .outerJoin(oldKnownFor)
      .map {
        case (userId, (newKnownForListOpt, oldKnownForListOpt)) =>
          val newKnownFor = newKnownForListOpt.getOrElse(Nil)
          val oldKnownFor = oldKnownForListOpt.getOrElse(Nil)

          if (newKnownFor.nonEmpty && oldKnownFor.isEmpty) {
            emptyToSomething.inc()
          }
          if (newKnownFor.isEmpty && oldKnownFor.nonEmpty) {
            somethingToEmpty.inc()
          }
          if (newKnownFor.isEmpty && oldKnownFor.isEmpty) {
            emptyToEmpty.inc()
          }

          if (newKnownFor.nonEmpty && oldKnownFor.nonEmpty) {
            val newClusterId = newKnownFor.head._1
            val oldClusterId = oldKnownFor.head._1

            if (newClusterId == oldClusterId) {
              sameCluster.inc()
            } else {
              diffCluster.inc()
            }
          }
          userId
      }
      .toIterableExecution

    Util.getCustomCountersString(calculateStatExec)
  }

  def compare(
    knownForA: TypedPipe[(Int, List[(Long, Float)])],
    knownForB: TypedPipe[(Int, List[(Long, Float)])],
    minSizeOfBiggerCluster: Int
  ): TypedPipe[(Int, Float)] = {
    knownForA
      .outerJoin(knownForB)
      .collect {
        case (clusterId, (membersInAOpt, membersInBOpt))
            if membersInAOpt.exists(_.size >= minSizeOfBiggerCluster) || membersInBOpt
              .exists(_.size >= minSizeOfBiggerCluster) =>
          val membersInA =
            membersInAOpt.map(_.toMap).getOrElse(Map.empty[Long, Float])
          val membersInB =
            membersInBOpt.map(_.toMap).getOrElse(Map.empty[Long, Float])
          (clusterId, cosine(membersInA, membersInB))
      }
  }

  def summarize(clusterToCosines: TypedPipe[(Int, Float)]): Execution[Option[Distribution]] = {
    clusterToCosines.values.map(x => List(x)).sum.toOptionExecution.map { listOpt =>
      listOpt.map { list => Util.distributionFromArray(list.map(_.toDouble).toArray) }
    }
  }
}

object CompareClustersAdhoc extends TwitterExecutionApp {
  implicit val tz: java.util.TimeZone = DateOps.UTC
  implicit val dp = DateParser.default

  def job: Execution[Unit] =
    Execution.getConfigMode.flatMap {
      case (config, mode) =>
        Execution.withId { implicit uniqueId =>
          val args = config.getArgs

          val knownForA = KnownForSources.transpose(KnownForSources.readKnownFor(args("knownForA")))
          val knownForB = KnownForSources.transpose(KnownForSources.readKnownFor(args("knownForB")))

          CompareClusters
            .compare(knownForA, knownForB, minSizeOfBiggerCluster = 10)
            .map { case (cId, cos) => "%d\t%.2f".format(cId, cos) }
            .writeExecution(TypedTsv(args("outputDir")))
        }
    }
}
