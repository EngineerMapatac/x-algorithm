import logging
import os

from grox.core.data_loaders.data_types import Post
from grox.core.schedules.types import TaskContext
from grox.core.tasks.task import TaskStopExecution, TaskWithPost
from grox.flows.mm_emb.adaptive_min_fav import AdaptiveMinFav
from grox.flows.mm_emb.constants import V8_SEARCH_MODEL_VERSION
from monitor.metrics import Metrics
from strato_http.queries.post_multimodal_embedding_mh_searchai import (
    StratoPostMultimodalEmbeddingMhSearchAi,
)

logger = logging.getLogger(__name__)

MIN_FAV_COUNT = 20_000

_TARGET_QPS = float(os.environ.get("MM_EMB_BACKFILL_TARGET_QPS", "5"))
_min_fav = AdaptiveMinFav(start=MIN_FAV_COUNT, target_qps=_TARGET_QPS)

_DROP_METRIC = "task.backfill_gate.dropped.count"


class TaskBackfillFavGate(TaskWithPost):
    @classmethod
    async def _exec_with_post(cls, ctx: TaskContext, post: Post) -> None:
        min_fav = _min_fav.threshold()
        Metrics.gauge("task.backfill_gate.min_fav").set(min_fav)
        if _min_fav.last_qps is not None:
            Metrics.gauge("task.backfill_gate.passed_qps").set(_min_fav.last_qps)

        fav_count = post.counts.likes if post.counts is not None else None
        if fav_count is None:
            Metrics.counter(_DROP_METRIC).add(
                1, attributes={"reason": "fav_count_missing"}
            )
            raise TaskStopExecution()

        if fav_count < min_fav:
            Metrics.counter(_DROP_METRIC).add(1, attributes={"reason": "low_favs"})
            raise TaskStopExecution()
        Metrics.counter("task.backfill_gate.passed.count").add(1)


class TaskBackfillPreloadGate(TaskWithPost):
    @classmethod
    async def _exec_with_post(cls, ctx: TaskContext, post: Post) -> None:
        post_id = int(post.id)

        existing = await StratoPostMultimodalEmbeddingMhSearchAi().fetch(
            post_id, V8_SEARCH_MODEL_VERSION
        )
        if existing is not None:
            Metrics.counter(_DROP_METRIC).add(
                1, attributes={"reason": "already_embedded"}
            )
            raise TaskStopExecution()

        _min_fav.observe_pass()
        Metrics.counter("task.backfill_gate.passed_preload.count").add(1)
