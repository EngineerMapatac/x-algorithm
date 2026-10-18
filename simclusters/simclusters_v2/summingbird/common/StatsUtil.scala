package com.twitter.simclusters_v2.summingbird.common

import com.twitter.summingbird.Counter
import com.twitter.summingbird.Group
import com.twitter.summingbird.Name
import com.twitter.summingbird.Platform
import com.twitter.summingbird.Producer
import com.twitter.summingbird.option.JobId

object StatsUtil {

  implicit class EnrichedProducer[P <: Platform[P], T](
    producer: Producer[P, T]
  )(
    implicit jobId: JobId) {
    def observe(counter: String): Producer[P, T] = {
      val stat = Counter(Group(jobId.get), Name(counter))
      producer.map { v =>
        stat.incr()
        v
      }
    }
  }
}
