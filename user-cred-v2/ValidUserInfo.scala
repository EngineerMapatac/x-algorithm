package com.twitter.health.platform_manipulation.user_cred_v2

import com.twitter.usersource.snapshot.flat.thriftscala.FlatUser

final case class ValidUserInfo(
  id: Long,
  isNearZero: Boolean,
  isBlueVerified: Boolean,
  isGrayVerified: Boolean,
  isGoldVerified: Boolean,
  isVerifiedOrg: Boolean,
  isVOAffiliate: Boolean,
) {
  def isPremium: Boolean =
    isBlueVerified || isGrayVerified || isGoldVerified || isVerifiedOrg || isVOAffiliate
}

object ValidUserInfo {
  private val NearZeroState = 1

  def fromFlatUser(flatUser: FlatUser): Option[ValidUserInfo] = {
    val ignoreUser = flatUser.restricted.contains(true) ||
      flatUser.deactivated.contains(true) ||
      flatUser.suspended.contains(true) ||
      flatUser.erased.contains(true)

    if (!ignoreUser) {
      flatUser.id.map { id =>
        ValidUserInfo(
          id = id,
          isNearZero = flatUser.userState.contains(NearZeroState),
          isBlueVerified = flatUser.isBlueVerified.getOrElse(false),
          isGrayVerified = flatUser.isGrayVerified.getOrElse(false),
          isGoldVerified = flatUser.isGoldVerified.getOrElse(false),
          isVerifiedOrg = flatUser.isVerifiedOrganization.getOrElse(false),
          isVOAffiliate = flatUser.isVerifiedOrganizationAffiliate.getOrElse(false),
        )
      }
    } else {
      None
    }
  }
}
