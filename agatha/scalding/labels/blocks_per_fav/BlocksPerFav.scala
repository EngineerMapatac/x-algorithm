package com.twitter.agatha.scalding.labels.blocks_per_fav

import com.twitter.hub.agatha.labels.LabelGenerator
import com.twitter.hub.agatha.thriftscala.UserLabel
import com.twitter.scalding.DateRange
import com.twitter.scalding.TypedPipe
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla

object BlocksPerFav extends LabelGenerator {
  override val labelName: String = "BlocksPerFav"

  override def read(dateRange: DateRange): TypedPipe[UserLabel] = {
    DAL
      .readMostRecentSnapshot(BlocksPerFavUserLabelScalaDataset, dateRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
  }
}

object MentionFilteredBlocksPerFav extends LabelGenerator {
  override val labelName: String = "MentionFilteredBlocksPerFav"

  override def read(dateRange: DateRange): TypedPipe[UserLabel] = {
    DAL
      .readMostRecentSnapshot(
        MentionFilteredBlocksPerFavUserLabelScalaDataset,
        dateRange
      )
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
  }
}
