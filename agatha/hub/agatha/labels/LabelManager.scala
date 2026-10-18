package com.twitter.hub.agatha.labels

import com.twitter.hub.agatha.thriftscala.UserLabel
import com.twitter.scalding.DateRange
import com.twitter.scalding.typed.TypedPipe

trait LabelManager {
  val labelGeneratorSet: Set[
    LabelGenerator
  ]

  private lazy val labelGenerators: Map[String, LabelGenerator] =
    labelGeneratorSet.map(lg => (lg.labelName, lg)).toMap

  def getLabels(labelName: String, dateRange: DateRange): TypedPipe[UserLabel] =
    labelGenerators.get(labelName).map(_.read(dateRange)).getOrElse(TypedPipe.empty)

  def getLabels(labelMap: Map[String, DateRange]): Map[String, TypedPipe[UserLabel]] = {
    labelMap.map { case (id, dateRange) => (id, getLabels(id, dateRange)) }
  }
}
