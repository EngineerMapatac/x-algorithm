package com.twitter.health.platform_manipulation.user_cred_v2

import com.twitter.data.proto.Flock

final case class Edge(sourceId: Long, destinationId: Long) {
  def asTuple: (Long, Long) =
    (sourceId, destinationId)
}

object Edge {
  private val ValidState = 0

  def fromFlockEdge(edge: Flock.Edge): Option[Edge] = {
    if (edge.getStateId == ValidState) {
      Some(Edge(sourceId = edge.getSourceId, destinationId = edge.getDestinationId))
    } else {
      None
    }
  }
}
