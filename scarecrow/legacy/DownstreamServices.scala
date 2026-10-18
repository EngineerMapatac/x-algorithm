package com.twitter.botmaker.app.scarecrow.legacy

import com.twitter.botmaker.DownstreamService

object DownstreamServices {

  case object ActionIntakeService extends DownstreamService {
    override val name: String = "ACTION_INTAKE_SERVICE"
  }

  case object Eventbus extends DownstreamService {
    override val name = "EVENTBUS"
  }

  case object Gizmoduck extends DownstreamService {
    override val name = "GIZMODUCK"
  }

  case object Guano extends DownstreamService {
    override val name = "GUANO"
  }

  case object LimiterService extends DownstreamService {
    override val name = "LIMITER_SERVICE"
  }

  case object ManhattanBotmakerHllCounterStore extends DownstreamService {
    override val name = "MANHATTAN_BOTMAKER_HLL_COUNTER_STORE"
  }

  case object ManhattanLabelstore extends DownstreamService {
    override val name = "MANHATTAN_LABELSTORE"
  }

  case object ManhattanUrlReputation extends DownstreamService {
    override val name = "MANHATTAN_URL_REPUTATION"
  }

  case object Pinkstore extends DownstreamService {
    override val name = "PINKSTORE"
  }

  case object SlugToTweetIdStore extends DownstreamService {
    override val name = "SLUG_TO_TWEET_ID_STORE"
  }

  case object SocialGraphService extends DownstreamService {
    override val name = "SOCIAL_GRAPH_SERVICE"
  }

  case object StratoGetUserCredV2 extends DownstreamService {
    override val name = "STRATO_GET_USER_CRED_V2"
  }

  case object StratoUserHealthScores extends DownstreamService {
    override val name: String = "STRATO_USER_HEALTH_SCORES"
  }

  case object StratoUserHealthSignals extends DownstreamService {
    override val name: String = "STRATO_USER_HEALTH_SIGNALS"
  }

  case object StratoSafetyLabels extends DownstreamService {
    override val name = "STRATO_SAFETY_LABELS"
  }

  case object StratoUserNsfaScore extends DownstreamService {
    override val name = "STRATO_USER_NSFA_SCORE"
  }

  case object Tweetypie extends DownstreamService {
    override val name = "TWEETYPIE"
  }

  case object UrlToSlugStore extends DownstreamService {
    override val name = "URL_TO_SLUG_STORE"
  }

  case object XReviewIntake extends DownstreamService {
    override val name = "XREVIEW_INTAKE"
  }
}
