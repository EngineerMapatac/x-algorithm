package com.twitter.agatha.scalding.labels

import com.twitter.agatha.scalding.labels.blocks_per_fav.BlocksPerFav
import com.twitter.agatha.scalding.labels.nsfw.NSFWLabel
import com.twitter.agatha.scalding.labels.reports_per_fav.ReportsPerFav
import com.twitter.agatha.scalding.labels.spam_suspended.SpamSuspendedLabel
import com.twitter.agatha.scalding.labels.spam_reports_per_fav.AllSpamReportsPerFav
import com.twitter.agatha.scalding.labels.spam_reports_per_fav.SpamReportsPerFav
import com.twitter.hub.agatha.labels.LabelGenerator
import com.twitter.hub.agatha.labels.LabelManager

object AgathaLabelManager extends LabelManager {
  override val labelGeneratorSet: Set[LabelGenerator] = Set(
    ReportsPerFav,
    AllSpamReportsPerFav,
    SpamReportsPerFav,
    BlocksPerFav,
    SpamSuspendedLabel,
    NSFWLabel,
  )
}
