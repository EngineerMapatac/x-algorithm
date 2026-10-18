from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass


@dataclass(frozen=True)
class Head:
    name: str
    index: int
    selection_weight: int
    flywheel_labels: tuple[str, ...]


HEADS: tuple[Head, ...] = (
    Head(
        "FollowBot",
        0,
        27_576,
        (
            "FOLLOW_UNFOLLOW_CYCLE",
            "PURE_FOLLOW_API_BURST",
            "API_ONLY_BOT",
            "GROWTH_SERVICE_BOT",
            "FOLLOW_FARM_BOT",
        ),
    ),
    Head(
        "LikeBot",
        1,
        60_320,
        (
            "PURE_LIKE_API_BURST",
            "LIKE_UNLIKE_CYCLE",
            "STEADY_LIKE_DRIP",
            "LIKE_FARM_BOT",
        ),
    ),
    Head(
        "EngagementAmplifier",
        2,
        5_496,
        (
            "LIKE_RETWEET_PAIR",
            "FOLLOW_THEN_FAV_PIPELINE",
            "FOLLOW_THEN_REPLY_PIPELINE",
            "REPLY_THEN_FOLLOW_PIPELINE",
            "ENGAGEMENT_AMPLIFIER",
            "FOLLOW_LIKE_AMPLIFIER",
            "OUTREACH_PIPELINE_BOT",
        ),
    ),
    Head(
        "ReplySpamBot",
        3,
        8_922,
        (
            "REPLY_SPAM_NO_CONSUMPTION",
            "REPLY_SPAM_BOT",
            "CONVERSATION_SPAMMER",
        ),
    ),
    Head(
        "TweetSpamBot",
        4,
        23_425,
        (
            "TWEET_CREATE_BURST",
            "QUOTE_TWEET_SPAMMER",
            "CONTENT_AMPLIFIER",
        ),
    ),
    Head(
        "RTBot",
        5,
        5_290,
        (
            "PURE_RETWEET_API_BURST",
            "RETWEET_UNRETWEET_CYCLE",
        ),
    ),
    Head(
        "MultiActionBot",
        6,
        17_474,
        (
            "MULTI_ACTION_ENGAGEMENT_BOT",
            "SCROLL_AND_LIKE",
            "MIXED_AUTOMATION",
            "SCROLL_BOT_NO_ENGAGEMENT",
            "MIXED_ENGAGEMENT_AUTOMATION",
            "SYNTHETIC_CONSUMER",
            "AUTOMATED_ENGAGEMENT_BOT",
            "PASSIVE_SURVEILLANCE_BOT",
            "COORDINATED_INAUTHENTIC_BOT",
        ),
    ),
    Head(
        "LegitimateUser",
        7,
        0,
        (
            "LEGITIMATE_ORGANIC_USER",
            "INFREQUENT_LEGITIMATE_USER",
            "POWER_USER_API_CLIENT",
            "NEW_ACCOUNT_ONBOARDING",
            "DEVELOPER_TESTING",
            "LEGITIMATE_USER",
        ),
    ),
)

N_HEADS = len(HEADS)
HEAD_NAMES: tuple[str, ...] = tuple(h.name for h in HEADS)
HEAD_INDEX: dict[str, int] = {h.name: h.index for h in HEADS}
LEGITIMATE_USER_INDEX = HEAD_INDEX["LegitimateUser"]
POSITIVE_HEAD_NAMES: tuple[str, ...] = tuple(h.name for h in HEADS if h.name != "LegitimateUser")

FLYWHEEL_LABEL_TO_HEAD: dict[str, int] = {
    label: h.index for h in HEADS for label in h.flywheel_labels
}


def registry_hash() -> str:
    canonical = json.dumps(
        [(h.name, h.index, h.selection_weight, list(h.flywheel_labels)) for h in HEADS],
        sort_keys=True,
    )
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()[:16]
