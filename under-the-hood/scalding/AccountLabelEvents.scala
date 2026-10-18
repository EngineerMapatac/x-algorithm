package com.twitter.visibility.under_the_hood

import com.twitter.gizmoduck.thriftscala.UserModification
import com.twitter.scalding.TypedPipe

object AccountLabelEvents {
  import UnderTheHoodCommon._

  type Event = (Long, Long, Boolean)

  private[under_the_hood] def seedEventsPipe(
    seed: TypedPipe[(Long, String, Long, Long)],
    testUserIds: Set[Long],
    asOfMs: Long
  ): TypedPipe[((Long, String), Event)] =
    seed.collect {
      case (userId, labelValue, _, expiresMs) if inScope(testUserIds, userId) =>
        ((userId, labelValue), (asOfMs, expiresMs, true))
    }

  private[under_the_hood] def deltaEventsPipe(
    raw: TypedPipe[UserModification],
    testUserIds: Set[Long],
    asOfMs: Long,
    windowEndMs: Long
  ): TypedPipe[((Long, String), Event)] =
    raw.flatMap { m =>
      val userId = m.userId.getOrElse(-1L)
      val ts = m.updatedAtMsec.getOrElse(0L)
      if (m.userId.isEmpty || !inScope(testUserIds, userId) || m.success.contains(false) ||
        ts <= asOfMs || ts >= windowEndMs)
        Iterable.empty[((Long, String), Event)]
      else {
        m.update
          .getOrElse(Nil).iterator.filter(_.fieldName == "labels.asJson").flatMap { d =>
            val events = for {
              beforeStr <- d.before
              afterStr <- d.after
              before <- GizmoduckLabelDiff.labelApplyAndExpireTimes(beforeStr)
              after <- GizmoduckLabelDiff.labelApplyAndExpireTimes(afterStr)
            } yield {
              val applied = after.keySet.map { name =>
                ((userId, name), (ts, after(name)._2, true))
              }
              val removed =
                (before.keySet -- after.keySet).map { name =>
                  ((userId, name), (ts, 0L, false))
                }
              applied ++ removed
            }
            events.getOrElse(Set.empty[((Long, String), Event)])
          }.toSeq
      }
    }
}
