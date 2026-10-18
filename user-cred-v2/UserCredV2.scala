package com.twitter.health.platform_manipulation.user_cred_v2

final case class UserCredV2(
  id: Long,
  mass: Double,
  score: Double,
)

object UserCredV2 {
  private val ScoreSlope = 7.07
  private val ScoreIntercept = 165.2

  def fromMass(id: Long, mass: Double): UserCredV2 = {
    val rawScore =
      if (mass <= 0) 0.0
      else ScoreIntercept + ScoreSlope * scala.math.log(mass)
    val score = (rawScore max 0.0) min 100.0
    UserCredV2(id = id, mass = mass, score = score)
  }
}
