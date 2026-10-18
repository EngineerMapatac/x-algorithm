package com.twitter.agatha.scalding.labels.nsfw

import com.twitter.hub.agatha.labels.LabelGenerator
import com.twitter.hub.agatha.labels.LabelManager

object NsfwAgathaLabelManager extends LabelManager {
  override val labelGeneratorSet: Set[LabelGenerator] = Set(NSFWLabel)
}
