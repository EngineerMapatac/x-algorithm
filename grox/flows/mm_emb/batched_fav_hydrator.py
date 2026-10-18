import asyncio
import logging
import os
import time

from strato_http.queries.favorite_counts_batched import StratoFavoriteCountsBatched

from grox.core.data_loaders.data_types import Counts
from grox.core.data_loaders.message_queue_loader import MessageQueuePayload
from monitor.metrics import Metrics

logger = logging.getLogger(__name__)

BATCH_SIZE = 100
MAX_CONCURRENT_RPCS = int(os.environ.get("MM_EMB_BACKFILL_FAV_RPC_CONCURRENCY", "4"))

_IDS_METRIC = "backfill_fav_hydrator.ids.count"
_RPC_METRIC = "backfill_fav_hydrator.rpc.count"


class BatchedFavHydrator:
    def __init__(self) -> None:
        self._client = StratoFavoriteCountsBatched()
        self._sem = asyncio.Semaphore(MAX_CONCURRENT_RPCS)

    async def hydrate(self, payloads: list[MessageQueuePayload]) -> None:
        chunks = [
            payloads[i : i + BATCH_SIZE] for i in range(0, len(payloads), BATCH_SIZE)
        ]
        await asyncio.gather(*[self._hydrate_chunk(c) for c in chunks])

    async def _hydrate_chunk(self, payloads: list[MessageQueuePayload]) -> None:
        favs = await self._fetch_favs([p.post.id for p in payloads])
        Metrics.histogram("backfill_fav_hydrator.batch_size").record(len(payloads))
        if favs is None:
            Metrics.counter(_IDS_METRIC).add(
                len(payloads), attributes={"result": "batch_failed"}
            )
            return
        for p in payloads:
            fav = favs.get(p.post.id)
            if fav is None:
                Metrics.counter(_IDS_METRIC).add(1, attributes={"result": "missing"})
            else:
                p.post.counts = Counts(likes=fav)
                Metrics.counter(_IDS_METRIC).add(1, attributes={"result": "hydrated"})

    async def _fetch_favs(self, post_ids: list[str]) -> dict[str, int | None] | None:
        async with self._sem:
            for attempt in ("success", "retried"):
                try:
                    start = time.monotonic()
                    favs = await self._client.fetch_favorite_counts(post_ids)
                    Metrics.histogram("backfill_fav_hydrator.rpc_latency").record(
                        time.monotonic() - start
                    )
                    Metrics.counter(_RPC_METRIC).add(1, attributes={"status": attempt})
                    return favs
                except Exception:
                    await asyncio.sleep(0.2)
            Metrics.counter(_RPC_METRIC).add(1, attributes={"status": "failed"})
            logger.warning(
                f"batch fav fetch failed twice, {len(post_ids)} ids left unhydrated and will be dropped by the fav gate as fav_count_missing"
            )
            return None
