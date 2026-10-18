import logging

from grox.core.data_loaders.data_types import Post
from grox.core.schedules.types import TaskContext
from grox.flows.ptos.state import SafetyPtosState
from monitor.metrics import Metrics
from strato_http.queries.safety_post_annotations_result import (
    StratoSafetyPostAnnotationsResultDirectMh,
)

logger = logging.getLogger(__name__)

_result_direct_mh = StratoSafetyPostAnnotationsResultDirectMh()


async def post_is_already_flagged_nsfw(ctx: TaskContext, post: Post) -> bool:
    state = ctx.state(SafetyPtosState)
    if state.prior_nsfw is None:
        state.prior_nsfw = await _fetch(post)
    return state.prior_nsfw


async def _fetch(post: Post) -> bool:
    try:
        result = await _result_direct_mh.fetch(int(post.id))
    except Exception as e:
        Metrics.counter("safety_ptos.prior_nsfw_lookup_error.count").add(1)
        logger.warning(
            f"Post {post.id}: NSFW MH lookup failed, treating as not flagged: {e}"
        )
        return False
    return bool(
        result and result.safetyBoolMetadata and result.safetyBoolMetadata.isNsfw
    )
