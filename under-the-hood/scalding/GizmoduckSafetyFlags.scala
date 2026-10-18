package com.twitter.visibility.under_the_hood

import com.twitter.gizmoduck.thriftscala.UserModification
import com.twitter.scalding.TypedPipe

object GizmoduckSafetyFlags {

  val NsfwAdminLabel = "NsfwAdmin"

  private val FieldNameToLabel = Map(
    "safety.nsfwAdmin" -> NsfwAdminLabel
  )

  type FlagDiff = (Long, Boolean, Boolean)

  private[under_the_hood] val UsersourceFlagColumns: Set[String] =
    Set("id", "nsfw_admin")

  private[under_the_hood] def usersourceFlags(
    u: com.twitter.usersource.snapshot.flat.thriftscala.FlatUser
  ): Map[String, Boolean] =
    Map(
      NsfwAdminLabel -> u.nsfwAdmin.getOrElse(false)
    )

  private def parseBool(s: String): Option[Boolean] =
    s.trim.toLowerCase match {
      case "true" | "1" => Some(true)
      case "false" | "0" => Some(false)
      case _ => None
    }

  private[under_the_hood] def flagDiffsPipe(
    raw: TypedPipe[UserModification],
    flagLabels: Set[String],
    minTsMs: Long
  ): TypedPipe[((Long, String), FlagDiff)] =
    raw.flatMap { m =>
      val ts = m.updatedAtMsec.getOrElse(0L)
      if (m.userId.isEmpty || m.success.contains(false) || ts < minTsMs)
        Iterable.empty[((Long, String), FlagDiff)]
      else {
        val userId = m.userId.get
        m.update
          .getOrElse(Nil).iterator.flatMap { d =>
            FieldNameToLabel.get(d.fieldName) match {
              case Some(label) if flagLabels.contains(label) =>
                for {
                  beforeStr <- d.before
                  afterStr <- d.after
                  before <- parseBool(beforeStr)
                  after <- parseBool(afterStr)
                } yield ((userId, label), (ts, before, after))
              case _ => None
            }
          }.toSeq
      }
    }
}
