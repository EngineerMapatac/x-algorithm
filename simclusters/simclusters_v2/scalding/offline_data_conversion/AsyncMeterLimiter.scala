package com.twitter.simclusters_v2.scalding.offline_data_conversion

import com.twitter.conversions.DurationOps._
import com.twitter.util.Future
import com.twitter.concurrent.AsyncMeter
import com.twitter.finagle.util.DefaultTimer
import com.twitter.util.Duration
import com.twitter.util.logging.Logging
import java.util.concurrent.RejectedExecutionException

class AsyncMeterLimiter(burstSize: Int, burstDuration: Duration = 1.second) extends Logging {

  private val meter = AsyncMeter.newUnboundedMeter(burstSize, burstDuration)(DefaultTimer)
  def run[T](permits: Int = 1, f: => Future[T]): Future[T] =
    meter
      .await(permits).handle {
        case t: RejectedExecutionException =>
          info(s"RejectedExecutionException: $t")
      }.before {
        f
      }
}
