package com.twitter.agatha.scalding.util

import com.twitter.agatha.thriftscala.FlattenedBlinkScore
import com.twitter.algebird.Aggregator
import com.twitter.dal.client.dataset.SnapshotDALDataset
import com.twitter.scalding.DateRange
import com.twitter.scalding.Execution
import com.twitter.scalding.RichDate
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla

object FeaturesToLabels {

  private[util] def takeN(pipe: TypedPipe[(Long, Double)], n: Int): TypedPipe[(Long, Int)] = {
    pipe
      .groupRandomly(10)
      .sortWith(_._2 < _._2)
      .reverse
      .take(n)
      .values
      .groupAll
      .sortWith(_._2 < _._2)
      .reverse
      .take(n)
      .mapValues(fbs => (fbs._1, fbs._2))
      .scanLeft((0L, 0)) { (prev, l) => (l._1, prev._2 + 1) }
      .values
  }

  def makeIntervals(list: List[Int], top: Boolean): List[(Int, Int)] = {
    val intervals = list.scanLeft((0, 0)) { (prev, end) => (prev._2, end) }.tail
    val result = if (top) {
      intervals.map {
        case (_, endRange) => (0, endRange)
      }
    } else intervals
    println(s"Created interval $result from input $list $top")
    result
  }

  def parseList[V](list: List[Int], outputList: List[V], top: Boolean): Map[(Int, Int), V] = {
    require(
      list.size == outputList.size,
      s"Error parsing threshold list $list with outputs $outputList: ${list.size} != ${outputList.size}")
    makeIntervals(list, top).zip(outputList).toMap
  }

  def readDatasetAndJoinOnActiveUsers(
    dataset: SnapshotDALDataset[FlattenedBlinkScore],
    dateRange: DateRange
  ): Execution[TypedPipe[FlattenedBlinkScore]] = {
    DAL
      .readMostRecentSnapshot(dataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .groupBy(_.userId)
      .join(GetActiveUsers(DateRange(dateRange.start, RichDate.now)).asKeys)
      .mapValues(_._1)
      .values
      .forceToDiskExecution
  }

  def generateCutoffs[V](
    percentagesByLabel: Map[String, List[Double]],
    top: Boolean,
    severityCategories: Map[String, List[V]],
    labelNames: List[String],
    labels: TypedPipe[FlattenedBlinkScore]
  ): Execution[Map[String, Map[(Int, Int), V]]] = {
    require(
      labelNames.length == percentagesByLabel.keys.toSeq.length,
      s"Inconsistent lengths, label names: $labelNames, $percentagesByLabel")
    Execution
      .sequence(labelNames.map {
        labelName =>
          labels
            .filter(_.label == labelName)
            .aggregate(Aggregator.size)
            .toIterableExecution
            .map { countIter =>
              val count = countIter.headOption.getOrElse(0L)
              println(s"Total $labelName count: $count")
              val cutoffLevels = percentagesByLabel(labelName)
                .map(percentage => (percentage * count).toInt)
              cutoffLevels.zipWithIndex.foreach {
                case (level, index) =>
                  println(s"$labelName level $index cutoff: $level")
              }
              Map(labelName -> parseList(cutoffLevels, severityCategories(labelName), top))
            }
      }).map(_.reduce(_ ++ _))
  }

  def applyCutOffs[V](
    labelsToScores: Execution[Map[String, Map[(Int, Int), V]]],
    flattenedBlinkScore: TypedPipe[FlattenedBlinkScore]
  ): Execution[TypedPipe[(String, (Long, V))]] = {
    labelsToScores.map { cutoffMap =>
      cutoffMap
        .map {
          case (label, labelCutoffs) =>
            takeN(
              flattenedBlinkScore.collect {
                case fbs if fbs.label == label => (fbs.userId, fbs.score)
              },
              labelCutoffs.keySet.max._2)
              .flatMap {
                case (id, rank) =>
                  labelCutoffs
                    .collect {
                      case ((start, end), labelValue) if rank > start && rank <= end => labelValue
                    }
                    .map { lv => (label, (id, lv)) }
              }
        }.reduce(_ ++ _)
    }
  }
}
