package com.twitter.health.platform_manipulation.user_cred_v2

import com.twitter.scalding.Execution
import com.twitter.scalding.TypedPipe

final case class GateMetrics(
  rows: Long,
  massSum: Double,
  vitCount: Long,
  gte90Count: Long,
)

final case class GateResult(
  name: String,
  detail: String,
  ok: Boolean,
)

object SnapshotSafeguard {

  def computeMetrics(
    userCredPipe: TypedPipe[UserCredV2],
    vitThreshold: Double,
  ): Execution[GateMetrics] = {
    userCredPipe
      .map { uc =>
        (
          1L,
          uc.mass,
          if (uc.score >= vitThreshold) 1L else 0L,
          if (uc.score >= 90.0) 1L else 0L,
        )
      }
      .sum
      .toOptionExecution
      .map {
        case Some((rows, massSum, vitCount, gte90Count)) =>
          GateMetrics(rows = rows, massSum = massSum, vitCount = vitCount, gte90Count = gte90Count)
        case None =>
          GateMetrics(rows = 0L, massSum = 0.0, vitCount = 0L, gte90Count = 0L)
      }
  }

  def evaluate(
    current: GateMetrics,
    previousOpt: Option[GateMetrics],
    config: UserCredV2Config,
  ): Seq[GateResult] = {
    val g1 = previousOpt.map { previous =>
      relativeCheck("G1_row_count", current.rows, previous.rows, config.gateRowCountMaxDelta)
    }
    val g2 = Some(
      GateResult(
        name = "G2_mass_sum",
        detail = f"massSum=${current.massSum}%.6f " +
          s"bounds=[${config.gateMassSumMin}, ${config.gateMassSumMax}]",
        ok = current.massSum >= config.gateMassSumMin && current.massSum <= config.gateMassSumMax,
      ))
    val g3 = previousOpt.map { previous =>
      relativeCheck(
        "G3_vit_count",
        current.vitCount,
        previous.vitCount,
        config.gateVitCountMaxDelta)
    }
    Seq(g1, g2, g3).flatten
  }

  private def relativeCheck(
    name: String,
    current: Long,
    previous: Long,
    maxDelta: Double,
  ): GateResult = {
    if (previous == 0L) {
      GateResult(
        name = name,
        detail = s"current=$current previous=0 (no valid baseline)",
        ok = current == 0L,
      )
    } else {
      val delta = math.abs(current.toDouble / previous.toDouble - 1.0)
      GateResult(
        name = name,
        detail = f"current=$current previous=$previous delta=$delta%.4f maxDelta=$maxDelta",
        ok = delta <= maxDelta,
      )
    }
  }
}
