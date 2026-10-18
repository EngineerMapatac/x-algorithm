from __future__ import annotations

import os

from monitor.metrics import Metrics
from redis_cli.dragonfly import DEFAULT_ENDPOINT, DragonflyCacheClient

DEDUP_ENDPOINT = os.environ.get("MM_EMB_BACKFILL_DEDUP_ENDPOINT", DEFAULT_ENDPOINT)
DEDUP_TTL_SECS = int(
    os.environ.get("MM_EMB_BACKFILL_DEDUP_TTL_SECS", str(3 * 24 * 3600))
)

_KEY_PREFIX = "groxbf:"


class BackfillConsumerDedup:
    def __init__(
        self,
        endpoint: str = DEDUP_ENDPOINT,
        ttl_secs: int = DEDUP_TTL_SECS,
    ) -> None:
        self._ttl_secs = ttl_secs
        self._client = DragonflyCacheClient(endpoint=endpoint)

    async def filter_first_seen(self, payloads: list) -> list:
        if not self._client.enabled or not payloads:
            return payloads
        wins = await self._client.set_nx_batch(
            [f"{_KEY_PREFIX}{p.post.id}" for p in payloads],
            "1",
            ttl_secs=self._ttl_secs,
        )
        winners = [p for p, won in zip(payloads, wins) if won is not False]
        passed = sum(1 for won in wins if won is True)
        errors = sum(1 for won in wins if won is None)
        duplicates = len(payloads) - passed - errors
        if passed:
            Metrics.counter("kafka_mm_backfill_dedup.count").add(
                passed, attributes={"result": "passed"}
            )
        if duplicates:
            Metrics.counter("kafka_mm_backfill_dedup.count").add(
                duplicates, attributes={"result": "duplicate"}
            )
        if errors:
            Metrics.counter("kafka_mm_backfill_dedup.count").add(
                errors, attributes={"result": "error"}
            )
        return winners
