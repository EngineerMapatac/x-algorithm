package com.twitter.agatha.scalding.blink

import com.twitter.bijection.NumericInjections
import com.twitter.dal.personal_data.thriftscala.PersonalDataType
import com.twitter.scalding_internal.multiformat.format.keyval.KeyValInjection
import com.twitter.scalding_internal.multiformat.format.keyval.KeyValInjection.Long2BigEndian
import com.twitter.scalding_internal.multiformat.format.keyval.KeyValInjection.genericInjection

object ManhattanImportInjection extends NumericInjections {
  val Injection = KeyValInjection(
    keyCodec = Long2BigEndian(Some(Set(PersonalDataType.UserId))),
    valueCodec = genericInjection(double2BigEndian, Some(Set(PersonalDataType.UserSafetyScores)))
  )
}
