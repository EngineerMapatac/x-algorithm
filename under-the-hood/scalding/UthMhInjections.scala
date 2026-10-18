package com.twitter.visibility.under_the_hood

import com.twitter.scalding_internal.multiformat.format.keyval.KeyValInjection
import com.twitter.scalding_internal.multiformat.format.keyval.KeyValInjection.Int2BigEndian
import com.twitter.scalding_internal.multiformat.format.keyval.KeyValInjection.ScalaBinaryThrift
import com.twitter.visibility.under_the_hood.thriftscala.UthReferenceMonthAggregate
import com.twitter.visibility.under_the_hood.thriftscala.UthUserMonthAggregate
import com.twitter.visibility.under_the_hood.thriftscala.UthUserMonthKey

object UthMhInjections {

  val UserMonthInjection: KeyValInjection[UthUserMonthKey, UthUserMonthAggregate] =
    KeyValInjection(
      keyCodec = ScalaBinaryThrift(UthUserMonthKey),
      valueCodec = ScalaBinaryThrift(UthUserMonthAggregate)
    )

  val ReferenceMonthInjection: KeyValInjection[Int, UthReferenceMonthAggregate] =
    KeyValInjection(
      keyCodec = Int2BigEndian,
      valueCodec = ScalaBinaryThrift(UthReferenceMonthAggregate)
    )
}
