package com.twitter.simclusters_v2.score

import com.twitter.simclusters_v2.thriftscala.{Score => ThriftScore}

case class Score(score: Double) {

  implicit lazy val toThrift: ThriftScore = {
    ThriftScore(score)
  }
}

object Score {

  implicit val fromThriftScore: ThriftScore => Score = { thriftScore => Score(thriftScore.score) }

}
