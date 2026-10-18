import logging
import time
from typing import Any

import numpy as np
from embed.embed_http import EmbeddingModelConfig as HttpModelConfig
from embed.embed_http import XaiEmbeddingClientHttp
from grox.config.config import grox_config
from grox.core.data_loaders.data_types import Post, User
from grox.flows.mm_emb.renderer_v8_search import V8SearchEmbedPostRenderer
from monitor.metrics import Metrics
from strato_http.queries.user_core import StratoUserCore
from grox.flows.mm_emb.constants import (
    RECSYS_V8_SEARCH_BACKFILL_EMBED,
    RECSYS_V8_SEARCH_EMBED,
)

logger = logging.getLogger(__name__)

MAX_FRAMES = 4


class MultimodalPostEmbedderV8Search:
    EMBED_MODEL_KEY = RECSYS_V8_SEARCH_EMBED

    def __init__(self) -> None:
        embed_config = grox_config.get_embedding_model(self.EMBED_MODEL_KEY)
        http_config = HttpModelConfig(
            model_name=embed_config.model_name,
            endpoint=embed_config.endpoint,
            text_max_len=embed_config.text_max_len,
            timeout_seconds=60.0,
        )
        self._client = XaiEmbeddingClientHttp(config=http_config)
        self._user_core = StratoUserCore()

    @staticmethod
    def _renormalize(embedding: np.ndarray) -> list[float]:
        norm = float(np.linalg.norm(embedding))
        if norm > 0:
            embedding = embedding / norm
        return embedding.tolist()

    async def _fetch_core_user(self, user: User | None) -> dict[str, Any] | None:
        if user is None or user.id is None:
            return None
        try:
            return await self._user_core.fetch(int(user.id))
        except Exception as e:
            Metrics.counter("post_embedding_v8_search.core_user_fetch_error").add(1)
            logger.warning(f"core.User fetch failed for {user.id}: {e}")
            return None

    async def embed(
        self,
        post: Post,
        **kwargs,
    ) -> tuple[list[tuple[str, str | bytes]], list[float]]:
        total_start = time.perf_counter()

        core_user = await self._fetch_core_user(post.user)

        render_start = time.perf_counter()
        text, images = V8SearchEmbedPostRenderer.render_for_embedding(
            post,
            core_user=core_user,
            max_frames=MAX_FRAMES,
        )
        render_duration_ms = (time.perf_counter() - render_start) * 1000
        Metrics.histogram("post_embedding_v8_search.render_duration_ms").record(
            render_duration_ms
        )

        document: list[tuple[str, str | bytes]] = [("text", text)]
        for img in images:
            document.append(("image", img))

        if not text and not images:
            logger.warning(f"Post {post.id} has no text or media content")
            return document, []

        encode_start = time.perf_counter()
        embedding = await self._client.embed_openai_async(
            text=text, images=images or None
        )
        encode_duration_ms = (time.perf_counter() - encode_start) * 1000
        Metrics.histogram("post_embedding_v8_search.encode_duration_ms").record(
            encode_duration_ms
        )

        result = self._renormalize(embedding)

        total_duration_ms = (time.perf_counter() - total_start) * 1000
        Metrics.histogram("post_embedding_v8_search.total_duration_ms").record(
            total_duration_ms
        )
        Metrics.counter("post_embedding_v8_search.image_count").add(len(images))
        Metrics.counter("post_embedding_v8_search.core_user_hit").add(
            1 if core_user else 0
        )

        logger.info(
            f"Embedding v8-search post={post.id}: total={total_duration_ms:.1f}ms "
            f"(render={render_duration_ms:.1f}ms, encode={encode_duration_ms:.1f}ms), "
            f"n_img={len(images)}, has_core_user={core_user is not None}, "
            f"text_len={len(text)}, dim={len(result)}"
        )

        return document, result


class MultimodalPostEmbedderV8SearchBackfill(MultimodalPostEmbedderV8Search):
    EMBED_MODEL_KEY = RECSYS_V8_SEARCH_BACKFILL_EMBED
