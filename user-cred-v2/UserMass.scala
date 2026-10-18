package com.twitter.health.platform_manipulation.user_cred_v2

final case class UserMass(
  id: Long,
  mass: Double,
) {
  def toAggregate: Double = mass
}

object UserMass {
  def calcInitMass(flatUser: ValidUserInfo): UserMass = {
    UserMass(id = flatUser.id, mass = 1.0)
  }
}
