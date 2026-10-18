from typing import override

from monitor.metrics import Metrics

from grox.core.data_loaders.data_types import Post, Video
from grox.core.lm.convo import Video as ConvoVideo
from grox.core.schedules.types import TaskContext
from grox.core.tasks.task_filters import TaskFilterWithPost

_METRIC_PREFIX = "task.safety_ptos_special_video_screen"


class TaskSpecialVideoScreen(TaskFilterWithPost):
    @classmethod
    def _technique_signals(cls, convo_video: ConvoVideo) -> dict[str, bool]:
        return {
            "motion_reveal": bool(convo_video.motion_reveal_frames),
        }

    @override
    @classmethod
    async def _eligible_with_post(cls, post: Post, ctx: TaskContext) -> bool:
        screened = 0
        hits: set[str] = set()
        media = list(post.media or [])
        if post.quoted_post and post.quoted_post.media:
            media.extend(post.quoted_post.media)
        for medium in media:
            if not isinstance(medium, Video) or not medium.convo_video:
                continue
            screened += 1
            hits.update(
                technique
                for technique, hit in cls._technique_signals(medium.convo_video).items()
                if hit
            )
        if screened == 0:
            Metrics.counter(f"{_METRIC_PREFIX}.skipped.count").add(
                1, attributes={"reason": "no_hydrated_video"}
            )
            return False
        Metrics.counter(f"{_METRIC_PREFIX}.screened.count").add(
            1,
            attributes={
                "hit": str(bool(hits)).lower(),
                "techniques": ",".join(sorted(hits)) or "none",
            },
        )
        return bool(hits)
