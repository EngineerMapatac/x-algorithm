import logging
import os
import time

import grpc
from monitor.metrics import Metrics
from protos.night_owl_search.night_owl_search_pb2 import SearchRequest, SearchResponse
from protos.night_owl_search.night_owl_search_pb2_grpc import NightOwlSearchServiceStub

logger = logging.getLogger(__name__)

_LATENCY_BUCKETS_SECONDS: tuple[float, ...] = (
    0.005,
    0.01,
    0.025,
    0.05,
    0.1,
    0.25,
    0.5,
    1.0,
    1.5,
    2.0,
    5.0,
)


class NightOwlClient:
    def __init__(self, endpoint: str, timeout: float = 2.0):
        self._endpoint = endpoint
        self._timeout = timeout
        self._channel: grpc.aio.Channel | None = None
        self._stub: NightOwlSearchServiceStub | None = None

    async def _ensure_stub(self) -> NightOwlSearchServiceStub:
        if self._channel is None:
            self._channel = grpc.aio.insecure_channel(
                self._endpoint,
                options=[
                    ("grpc.max_receive_message_length", 128 * 1024 * 1024),
                    ("grpc.max_send_message_length", 128 * 1024 * 1024),
                    ("grpc.keepalive_time_ms", 30000),
                    ("grpc.keepalive_timeout_ms", 5000),
                    ("grpc.keepalive_permit_without_calls", True),
                ],
            )
            self._stub = NightOwlSearchServiceStub(self._channel)
        return self._stub

    async def search(
        self,
        request: SearchRequest,
        timeout: float | None = None,
    ) -> SearchResponse:
        stub = await self._ensure_stub()
        start = time.perf_counter()
        status = "OK"
        try:
            return await stub.Search(request, timeout=timeout or self._timeout)
        except grpc.aio.AioRpcError as e:
            status = e.code().name if e.code() is not None else "UNKNOWN"
            raise
        except Exception:
            status = "UNKNOWN"
            raise
        finally:
            try:
                Metrics.histogram(
                    "nightowl_client.search.duration",
                    explicit_bucket_boundaries_advisory=_LATENCY_BUCKETS_SECONDS,
                ).record(
                    time.perf_counter() - start,
                    {"status": status, "pid": str(os.getpid())},
                )
            except Exception:
                logger.exception("failed to record nightowl_client.search.duration")

    async def close(self) -> None:
        if self._channel is not None:
            await self._channel.close()
            self._channel = None
            self._stub = None
