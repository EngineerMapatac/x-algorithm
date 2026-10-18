import asyncio
import logging
import os
import uuid

from pydantic import BaseModel

from grox.config.config import grox_config
from grox.core.data_loaders.data_types import Post
from grox.core.data_loaders.message_queue_loader import MessageQueuePayload
from grox.core.schedules.types import TaskContext, TaskPayload
from grox.core.tasks.task import Task, TaskStopExecution
from grox.flows.mm_emb.backfill_consumer_dedup import BackfillConsumerDedup
from grox.flows.mm_emb.batched_fav_hydrator import BatchedFavHydrator
from grox.flows.mm_emb.plan_post_embedding_v8_search_backfill import (
    PlanPostEmbeddingV8SearchBackfill,
)
from monitor.metrics import Metrics

logger = logging.getLogger(__name__)

FANOUT_CONCURRENCY = int(os.environ.get("MM_EMB_BACKFILL_FANOUT_CONCURRENCY", "240"))

_METRIC = "task.backfill_fanout.posts.count"

_plan = PlanPostEmbeddingV8SearchBackfill()
_dedup = BackfillConsumerDedup()
_hydrator = BatchedFavHydrator()
_semaphore = asyncio.Semaphore(FANOUT_CONCURRENCY)
_inflight: set[asyncio.Task] = set()


class BackfillIdBatch(BaseModel):
    post_ids: list[int]


class TaskBackfillBatchFanout(Task):
    @classmethod
    async def _run_one(cls, payload: MessageQueuePayload) -> None:
        try:
            result = await asyncio.wait_for(
                _plan.execute(
                    TaskPayload(
                        payload_id=payload.mid,
                        post=payload.post,
                        plans={PlanPostEmbeddingV8SearchBackfill.KEY},
                    )
                ),
                timeout=grox_config.engine.task_timeout,
            )
            success = bool(result and result.success)
        except TimeoutError:
            logger.error(
                f"backfill fanout for post {payload.post.id} timed out after {grox_config.engine.task_timeout}s"
            )
            success = False
        except Exception:
            logger.exception(
                f"backfill fanout for post {payload.post.id} failed outside plan execution"
            )
            success = False
        finally:
            _semaphore.release()
        Metrics.counter(_METRIC).add(
            1, attributes={"result": "executed" if success else "failed"}
        )

    @classmethod
    async def _exec(cls, ctx: TaskContext) -> None:
        batch = ctx.payload.ext(BackfillIdBatch)
        if batch is None or not batch.post_ids:
            raise TaskStopExecution()
        payloads = [
            MessageQueuePayload(mid=uuid.uuid4().hex, post=Post(id=str(post_id)))
            for post_id in batch.post_ids
        ]
        first_seen = await _dedup.filter_first_seen(payloads)
        if not first_seen:
            return
        await _hydrator.hydrate(first_seen)
        for payload in first_seen:
            await _semaphore.acquire()
            task = asyncio.create_task(cls._run_one(payload))
            _inflight.add(task)
            task.add_done_callback(_inflight.discard)
