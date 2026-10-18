package com.twitter.health.platform_manipulation.user_cred_v2

final case class UserNode(
  id: Long,
  neighbourIds: Seq[Long],
  isNearZero: Boolean,
)
