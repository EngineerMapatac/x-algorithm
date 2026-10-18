import logging
from typing import override

from monitor.metrics import Metrics

from grox.config.config import grox_config
from grox.core.data_loaders.data_types import Post, Video
from grox.core.schedules.types import TaskContext
from grox.core.tasks.task_filters import TaskFilterWithPost
from grox.flows.ptos.prior_nsfw import post_is_already_flagged_nsfw

logger = logging.getLogger(__name__)

_METRIC_PREFIX = (
    "task.safety_ptos_media_injected_adult_infrared_video_spam_detection_filter"
)


def post_videos(post: Post) -> list[Video]:
    media = [
        *(post.media or []),
        *(post.quoted_post.media or [] if post.quoted_post else []),
    ]
    return [m for m in media if isinstance(m, Video)]


class TaskSafetyPtosMediaInjectedAdultInfraredVideoSpamDetectionFilter(
    TaskFilterWithPost
):
    @override
    @classmethod
    async def _eligible_with_post(cls, post: Post, ctx: TaskContext) -> bool:
        if not post.user:
            return cls._skip(post, "no_user")
        videos = post_videos(post)
        if not videos:
            return cls._skip(post, "no_video")
        if (
            post.get_fav_count()
            < grox_config.media_hydration.deluxe_fav_count_threshold
        ):
            return cls._skip(post, "not_high_fav")
        if await post_is_already_flagged_nsfw(ctx, post):
            return cls._skip(post, "already_nsfw")

        Metrics.counter(f"{_METRIC_PREFIX}.eligible.count").add(1)
        return True

    @classmethod
    def _skip(cls, post: Post, reason: str) -> bool:
        Metrics.counter(f"{_METRIC_PREFIX}.skipped.count").add(
            1, attributes={"reason": reason}
        )
        logger.info(f"Post {post.id}: skipped ({reason})")
        return False
