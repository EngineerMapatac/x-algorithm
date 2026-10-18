package com.twitter.health.platform_manipulation.user_cred_v2

import com.twitter.scalding.Args

final case class UserCredV2Config(
  outputPath: String,
  mhOutputPath: Option[String],
  jumpProbability: Double,
  diffThreshold: Double,
  maxIterations: Int,
  curIteration: Int,
  firstSnapshot: Boolean,
  engagementTeleportBeta: Double,
  vitThreshold: Double,
  gateRowCountMaxDelta: Double,
  gateVitCountMaxDelta: Double,
  gateMassSumMin: Double,
  gateMassSumMax: Double,
  skipGate: Boolean,
) {
  def nextIteration: UserCredV2Config = this.copy(curIteration = curIteration + 1)
}

object UserCredV2Config {
  val EngagementWindowDays: Int = 7

  def fromArgs(args: Args): UserCredV2Config = {
    val outputPath = args("output_path")
    val mhOutputPath = args.optional("mh_output_path")
    val jumpProbability = args.getOrElse("jump_prob", "0.2").toDouble
    val diffThreshold = args.getOrElse("threshold", "0.001").toDouble
    val maxIterations = args.getOrElse("max_iterations", "50").toInt
    val curIteration = args.getOrElse("cur_iteration", "0").toInt
    val firstSnapshot = args.boolean("first_snapshot")
    val engagementTeleportBeta = args.getOrElse("engagement_teleport_beta", "0.5").toDouble
    require(
      engagementTeleportBeta >= 0.0 && engagementTeleportBeta <= 1.0,
      s"engagement_teleport_beta must be in [0, 1], got $engagementTeleportBeta")

    val vitThreshold = args.getOrElse("vit_threshold", "55.0").toDouble
    val gateRowCountMaxDelta = args.getOrElse("gate_row_delta", "0.05").toDouble
    val gateVitCountMaxDelta = args.getOrElse("gate_vit_delta", "0.05").toDouble
    val gateMassSumMin = args.getOrElse("gate_mass_min", "0.99").toDouble
    val gateMassSumMax = args.getOrElse("gate_mass_max", "1.005").toDouble
    val skipGate = args.boolean("skip_gate")

    UserCredV2Config(
      outputPath = outputPath,
      mhOutputPath = mhOutputPath,
      jumpProbability = jumpProbability,
      diffThreshold = diffThreshold,
      maxIterations = maxIterations,
      curIteration = curIteration,
      firstSnapshot = firstSnapshot,
      engagementTeleportBeta = engagementTeleportBeta,
      vitThreshold = vitThreshold,
      gateRowCountMaxDelta = gateRowCountMaxDelta,
      gateVitCountMaxDelta = gateVitCountMaxDelta,
      gateMassSumMin = gateMassSumMin,
      gateMassSumMax = gateMassSumMax,
      skipGate = skipGate,
    )
  }
}
