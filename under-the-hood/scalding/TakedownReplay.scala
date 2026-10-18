package com.twitter.visibility.under_the_hood

import com.twitter.visibility.under_the_hood.TakedownLabels.Reason
import com.twitter.visibility.under_the_hood.TakedownLabels.ReasonType

object TakedownReplay {

  type State = (Long, Seq[Reason])

  case class PostLabel(
    label: String,
    carried: Int,
    removed: Int,
    firstWithheldMs: Option[Long],
    withheldAtAsOf: Boolean)

  private[under_the_hood] def effectiveStates(
    observations: Seq[TakedownObservation]
  ): Seq[State] = {
    var withholding: Set[Reason] = Set.empty
    var dmcaMedia: Set[Long] = Set.empty
    observations
      .groupBy(_.eventMs)
      .toSeq
      .sortBy { case (eventMs, _) => eventMs }
      .map {
        case (eventMs, batch) =>
          val states = batch.collect { case s: TakedownState => s }
          if (states.nonEmpty) withholding = states.flatMap(_.reasons).toSet
          batch.collect { case d: TakedownDelta => d }.sortBy(_.withheld).foreach { d =>
            d.mediaId match {
              case Some(mediaId) => if (d.withheld) dmcaMedia += mediaId else dmcaMedia -= mediaId
              case None => if (d.withheld) withholding += d.reason else withholding -= d.reason
            }
          }
          val state =
            (if (dmcaMedia.isEmpty) withholding else withholding + TakedownLabels.MediaDmca).toSeq
              .sortBy(_.sortKey)
          (eventMs, state)
      }
  }

  private def countries(state: Seq[Reason], t: ReasonType): Seq[String] =
    state.collect { case Reason(`t`, Some(cc)) => cc }

  private[under_the_hood] def postLabels(states: Seq[State]): Seq[PostLabel] = {
    val sorted = states.sortBy(_._1)
    val last = sorted.lastOption.map(_._2).getOrElse(Nil)
    val reasonTypes = sorted.flatMap(_._2).map(_.reasonType).distinct.sortBy(_.order)
    reasonTypes.flatMap { t =>
      val hasT = (s: Seq[Reason]) => s.exists(_.reasonType == t)
      val labelsOverTime = sorted
        .dropWhile { case (_, s) => !hasT(s) }
        .scanLeft(Seq.empty[String]) {
          case (prev, (_, s)) => if (hasT(s)) countries(s, t) else prev
        }
        .tail
        .map(TakedownLabels.label(t, _))
      val current = labelsOverTime.last
      val withheldNow = hasT(last)
      val currentRow = PostLabel(
        label = current,
        carried = 1,
        removed = if (withheldNow) 0 else 1,
        firstWithheldMs = sorted.collectFirst { case (ms, s) if hasT(s) => ms },
        withheldAtAsOf = withheldNow
      )
      val superseded = labelsOverTime.distinct.filterNot(_ == current).map { l =>
        PostLabel(l, carried = 0, removed = 0, firstWithheldMs = None, withheldAtAsOf = false)
      }
      currentRow +: superseded
    }
  }

  private[under_the_hood] def postLabelsAsOf(
    observations: Seq[TakedownObservation],
    asOfEndMs: Long
  ): Seq[PostLabel] =
    postLabels(effectiveStates(observations.filter(_.eventMs < asOfEndMs)))
}
