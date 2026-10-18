from typing import override

from monitor.metrics import Metrics

from grox.config.config import grox_config
from grox.core.data_loaders.data_types import Post, Video
from grox.core.schedules.types import TaskContext
from grox.core.tasks.task_filters import TaskFilterWithPost

_METRIC_PREFIX = "task.safety_ptos_special_video_filter"


class TaskSafetyPtosSpecialVideoFilter(TaskFilterWithPost):
    @override
    @classmethod
    async def _eligible_with_post(cls, post: Post, ctx: TaskContext) -> bool:
        reason = cls._skip_reason(post)
        if reason is not None:
            Metrics.counter(f"{_METRIC_PREFIX}.skipped.count").add(
                1, attributes={"reason": reason}
            )
            return False
        Metrics.counter(f"{_METRIC_PREFIX}.eligible.count").add(1)
        return True

    @classmethod
    def _skip_reason(cls, post: Post) -> str | None:
        if not post.user:
            return "no_user"
        media = list(post.media or [])
        if post.quoted_post and post.quoted_post.media:
            media.extend(post.quoted_post.media)
        if not any(isinstance(medium, Video) for medium in media):
            return "no_video"
        if (
            post.get_fav_count()
            < grox_config.media_hydration.deluxe_fav_count_threshold
        ):
            return "not_high_fav"
        return None
