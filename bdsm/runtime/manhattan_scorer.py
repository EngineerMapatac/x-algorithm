import logging
import os

import numpy as np

log = logging.getLogger("manhattan_scorer")


class ManhattanArrowReader:
    def __init__(self, endpoint: str = "localhost:8080"):
        from manhattan_cli.mh_client import ManhattanClient, ManhattanClientConfig, Tenant

        self.client = ManhattanClient(ManhattanClientConfig(endpoint=endpoint), secure=False)
        self.tenant = Tenant(
            cluster=os.environ.get("MH_CLUSTER", "your-cluster"),
            app_id=os.environ.get("MH_APP_ID", "your-app"),
            dataset=os.environ.get("MH_DATASET", "your-dataset"),
        )
        self._long_to_bytes = ManhattanClient.long_to_bytes

    def get_raw_bytes(self, user_id: int) -> bytes | None:
        try:
            import asyncio

            key_bytes = self._long_to_bytes(user_id)
            result = self.client.get(self.tenant, [key_bytes], [])
            if asyncio.iscoroutine(result):
                loop = asyncio.new_event_loop()
                try:
                    result = loop.run_until_complete(result)
                finally:
                    loop.close()
            if result is not None:
                return result.value
            return None
        except Exception as e:
            self._error_count = getattr(self, "_error_count", 0) + 1
            if self._error_count <= 5:
                log.warning(f"Manhattan get failed for user {user_id}: {type(e).__name__}: {e}")
            return None

    def get_raw_bytes_batch(
        self, user_ids: list[int], timeout: float = 10.0
    ) -> dict[int, bytes | None]:
        if not user_ids:
            return {}
        try:
            keys = [[self._long_to_bytes(uid)] for uid in user_ids]
            items = self.client.batch_get(self.tenant, keys, timeout=timeout)
            out: dict[int, bytes | None] = {}
            for uid, item in zip(user_ids, items):
                out[uid] = item.value if item is not None else None
            return out
        except Exception as e:
            self._error_count = getattr(self, "_error_count", 0) + 1
            if self._error_count <= 5:
                log.warning(
                    f"Manhattan batch_get failed for {len(user_ids)} users: {type(e).__name__}: {e}"
                )
            return {uid: None for uid in user_ids}


def _merge_single_user_batches(batches: list[dict], seq_len: int) -> dict:
    B = len(batches)
    if B == 1:
        return batches[0]

    def _concat(arrays):
        return np.concatenate(arrays, axis=0)

    return {
        "user_ids_raw": _concat([b["user_ids_raw"] for b in batches]),
        "tweet_ids_raw": _concat([b["tweet_ids_raw"] for b in batches]),
        "author_ids_raw": _concat([b["author_ids_raw"] for b in batches]),
        "action_seq": {
            key: _concat([b["action_seq"][key] for b in batches])
            for key in batches[0]["action_seq"]
        },
        "user_features": {
            key: _concat([b["user_features"][key] for b in batches])
            for key in batches[0]["user_features"]
        },
        "label_vector": _concat([b["label_vector"] for b in batches]),
        "label_mask": _concat([b["label_mask"] for b in batches]),
    }
