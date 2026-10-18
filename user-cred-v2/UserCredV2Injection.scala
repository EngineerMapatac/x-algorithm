package com.twitter.health.platform_manipulation.user_cred_v2

import com.twitter.dal.personal_data.thriftscala.PersonalDataType
import com.twitter.scalding_internal.multiformat.format.keyval.KeyValInjection
import com.twitter.scalding_internal.multiformat.format.keyval.KeyValInjection.Long2BigEndian
import com.twitter.scalding_internal.multiformat.format.keyval.KeyValInjection.ScalaBinaryThrift

object UserCredV2Injection {
  val Injection: KeyValInjection[Long, thriftscala.UserCredV2Scores] = KeyValInjection(
    keyCodec = Long2BigEndian(Some(Set(PersonalDataType.UserId))),
    valueCodec = ScalaBinaryThrift(thriftscala.UserCredV2Scores),
  )
}
