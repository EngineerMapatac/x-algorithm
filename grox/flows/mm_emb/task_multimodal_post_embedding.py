import logging

from grox.core.tasks.task import TaskWithPost, TaskStopExecution
from monitor.metrics import Metrics
from grox.core.schedules.types import TaskContext
from grox.flows.mm_emb.state import MultimodalPostEmbeddingState
from grox.core.data_loaders.data_types import Post, Video
from grox.flows.mm_emb.embedder import MultimodalPostEmbedderV5
from grox.flows.mm_emb.embedder_v82 import MultimodalPostEmbedderV82
from grox.flows.mm_emb.embedder_v8_search import (
    MultimodalPostEmbedderV8Search,
    MultimodalPostEmbedderV8SearchBackfill,
)
from grox.flows.mm_emb.constants import V8_SEARCH_STATE_KEY

logger = logging.getLogger(__name__)


class TaskMultimodalPostEmbeddingV5(TaskWithPost):
    embedder = MultimodalPostEmbedderV5()

    @classmethod
    async def _exec_with_post(cls, ctx: TaskContext, post: Post) -> None:
        try:
            transcripts = []
            if post.media:
                for m in post.media:
                    if (
                        isinstance(m, Video)
                        and m.convo_video
                        and m.convo_video.asr_transcript
                    ):
                        transcripts.append(m.convo_video.asr_transcript)
            transcript = "\n".join(transcripts) if transcripts else None
            _, embedding = await cls.embedder.embed(post, transcript=transcript)
        except Exception as e:
            Metrics.counter("task.multimodal_post_embedding_v5.error").add(1)
            logger.warning(
                f"TaskMultimodalPostEmbeddingV5 failed for post {post.id}: {e}"
            )
            raise
        if not embedding:
            logger.warning(
                f"TaskMultimodalPostEmbeddingV5 skipping post {post.id}: no valid embedding returned"
            )
            Metrics.counter("task.multimodal_post_embedding_v5.skipped").add(1)
            raise TaskStopExecution(f"No valid embedding for post {post.id}")
        ctx.state(MultimodalPostEmbeddingState).embeddings["v5_1"] = embedding
        logger.info(
            f"TaskMultimodalPostEmbeddingV5 Embedding Added, length: {len(embedding)}, has_transcript={transcript is not None}"
        )
        Metrics.counter("task.multimodal_post_embedding_v5.count").add(1)
        if transcript:
            Metrics.counter(
                "task.multimodal_post_embedding_v5.with_transcript.count"
            ).add(1)


class TaskMultimodalPostEmbeddingV82(TaskWithPost):
    embedder = MultimodalPostEmbedderV82()

    @classmethod
    async def _exec_with_post(cls, ctx: TaskContext, post: Post) -> None:
        try:
            _, embedding = await cls.embedder.embed(post)
        except Exception as e:
            Metrics.counter("task.multimodal_post_embedding_v82.error").add(1)
            logger.warning(
                f"TaskMultimodalPostEmbeddingV82 failed for post {post.id}: {e}"
            )
            raise
        if not embedding:
            Metrics.counter("task.multimodal_post_embedding_v82.empty").add(1)
            logger.info(
                f"TaskMultimodalPostEmbeddingV82 produced empty embedding for post {post.id}"
            )
            raise TaskStopExecution(f"Empty v8.2 embedding for post {post.id}")
        ctx.state(MultimodalPostEmbeddingState).embeddings["v8_2"] = embedding
        logger.info(
            f"TaskMultimodalPostEmbeddingV82 Embedding Added, length: {len(embedding)}"
        )
        Metrics.counter("task.multimodal_post_embedding_v82.count").add(1)


class TaskMultimodalPostEmbeddingV8Search(TaskWithPost):
    embedder = MultimodalPostEmbedderV8Search()

    @classmethod
    async def _exec_with_post(cls, ctx: TaskContext, post: Post) -> None:
        try:
            _, embedding = await cls.embedder.embed(post)
        except Exception as e:
            Metrics.counter("task.multimodal_post_embedding_v8_search.error").add(1)
            logger.warning(
                f"TaskMultimodalPostEmbeddingV8Search failed for post {post.id}: {e}"
            )
            raise
        if not embedding:
            Metrics.counter("task.multimodal_post_embedding_v8_search.empty").add(1)
            logger.info(
                f"TaskMultimodalPostEmbeddingV8Search produced empty embedding for post {post.id}"
            )
            raise TaskStopExecution(f"Empty v8-search embedding for post {post.id}")
        ctx.state(MultimodalPostEmbeddingState).embeddings[V8_SEARCH_STATE_KEY] = (
            embedding
        )
        logger.info(
            f"TaskMultimodalPostEmbeddingV8Search Embedding Added, length: {len(embedding)}"
        )
        Metrics.counter("task.multimodal_post_embedding_v8_search.count").add(1)


class TaskMultimodalPostEmbeddingV8SearchBackfill(TaskWithPost):
    embedder = MultimodalPostEmbedderV8SearchBackfill()

    @classmethod
    async def _exec_with_post(cls, ctx: TaskContext, post: Post) -> None:
        try:
            _, embedding = await cls.embedder.embed(post)
        except Exception:
            Metrics.counter("task.multimodal_post_embedding_v8_search.error").add(1)
            raise
        if not embedding:
            Metrics.counter("task.multimodal_post_embedding_v8_search.empty").add(1)
            raise TaskStopExecution(f"Empty v8-search embedding for post {post.id}")
        ctx.state(MultimodalPostEmbeddingState).embeddings[V8_SEARCH_STATE_KEY] = (
            embedding
        )
        Metrics.counter("task.multimodal_post_embedding_v8_search.count").add(1)
