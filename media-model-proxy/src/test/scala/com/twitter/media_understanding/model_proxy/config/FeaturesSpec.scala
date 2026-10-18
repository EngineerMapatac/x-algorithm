package com.twitter.media_understanding.model_proxy.config

import com.twitter.decider.NullDecider
import org.scalatest.matchers.should.Matchers
import org.scalatest.wordspec.AnyWordSpec

class FeaturesSpec extends AnyWordSpec with Matchers {
  "Features" should {
    "validate" in {
      val features = new Features(new NullDecider, useDevelFeatureConfig = false)
      features.validate()
    }
  }
}
