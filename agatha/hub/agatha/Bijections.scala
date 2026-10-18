package com.twitter.hub.agatha

import com.twitter.bijection.Injection
import com.twitter.bijection.scrooge.BinaryScalaCodec
import com.twitter.hub.agatha.thriftscala.FeatureIdentifier

object Bijections {
  implicit val serializeFeature = BinaryScalaCodec(FeatureIdentifier).toFunction
  implicit val serializeLabel = implicitly[Injection[String, Array[Byte]]].toFunction
}
