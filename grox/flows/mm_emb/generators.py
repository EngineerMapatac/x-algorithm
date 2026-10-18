import struct
import uuid

from kafka_cli.config import KafkaMessage
from monitor.metrics import Metrics
from typing import override

from grox.core.data_loaders.kafka_loader import KafkaLoader, KafkaPostLoader
from grox.core.data_loaders.message_queue_loader import MessageQueuePayload
from grox.core.generators.stream_generator import StreamTaskGenerator
from grox.flows.mm_emb.batched_fav_hydrator import BATCH_SIZE
from grox.flows.mm_emb.plan_post_embedding_v5 import PlanPostEmbeddingV5
from grox.flows.mm_emb.plan_post_embedding_v5_for_reply import (
    PlanPostEmbeddingV5ForReply,
)
from grox.flows.mm_emb.plan_post_embedding_v82 import PlanPostEmbeddingV82
from grox.flows.mm_emb.plan_post_embedding_v82_for_reply import (
    PlanPostEmbeddingV82ForReply,
)
from grox.flows.mm_emb.plan_post_embedding_v8_search import PlanPostEmbeddingV8Search
from grox.flows.mm_emb.plan_post_embedding_v8_search_backfill_batch import (
    PlanPostEmbeddingV8SearchBackfillBatch,
)
from grox.flows.mm_emb.task_backfill_fanout import BackfillIdBatch
from grox.flows.mm_emb.plan_post_embedding_v8_search_for_reply import (
    PlanPostEmbeddingV8SearchForReply,
)
from grox.core.registry import register
from grox.flows.mm_emb.constants import (
    POST_EMBEDDING_BACKFILL_STREAM,
    POST_EMBEDDING_V5_FOR_REPLY_STREAM,
    POST_EMBEDDING_V5_RECOVERY_STREAM,
    POST_EMBEDDING_V5_STREAM,
    POST_EMBEDDING_V8_2_FOR_REPLY_STREAM,
    POST_EMBEDDING_V8_2_RECOVERY_STREAM,
    POST_EMBEDDING_V8_2_STREAM,
    POST_EMBEDDING_V8_SEARCH_FOR_REPLY_STREAM,
    POST_EMBEDDING_V8_SEARCH_STREAM,
    TOPIC_BACKFILL,
    TOPIC_MIN_TRACTION_MULTI_MODAL,
    TOPIC_REQUESTS_WITH_SUMMARY,
    TOPIC_V5_RECOVERY,
    TOPIC_V8_RECOVERY,
)


@register
class PostEmbeddingV5StreamTaskGenerator(StreamTaskGenerator):
    TASK_GENERATOR_TYPE = POST_EMBEDDING_V5_STREAM
    PLANS_TO_INJECT = {PlanPostEmbeddingV5.KEY}

    def _get_loader(self):
        return KafkaPostLoader(TOPIC_REQUESTS_WITH_SUMMARY)


@register
class PostEmbeddingV5RecoveryStreamTaskGenerator(StreamTaskGenerator):
    TASK_GENERATOR_TYPE = POST_EMBEDDING_V5_RECOVERY_STREAM
    PLANS_TO_INJECT = {PlanPostEmbeddingV5.KEY}

    def _get_loader(self):
        return KafkaPostLoader(TOPIC_V5_RECOVERY)


class KafkaMmBackfillRequestLoader(KafkaLoader):
    def __init__(self, topic_name: str):
        super().__init__(topic_name)

    @override
    def _messages_to_payloads(
        self, messages: list[KafkaMessage]
    ) -> list[MessageQueuePayload]:
        post_ids: list[int] = []
        for message in messages:
            value = message.value
            if not value or len(value) % 8 != 0:
                Metrics.counter("kafka_mm_backfill_loader.skipped.count").add(
                    1, attributes={"reason": "bad_value_len"}
                )
                continue
            for (post_id,) in struct.iter_unpack("<q", value):
                if post_id <= 0:
                    Metrics.counter("kafka_mm_backfill_loader.skipped.count").add(
                        1, attributes={"reason": "invalid_post_id"}
                    )
                    continue
                post_ids.append(post_id)
        payloads: list[MessageQueuePayload] = []
        for offset in range(0, len(post_ids), BATCH_SIZE):
            payload = MessageQueuePayload(mid=uuid.uuid4().hex)
            payload.set_ext(
                BackfillIdBatch(post_ids=post_ids[offset : offset + BATCH_SIZE])
            )
            payloads.append(payload)
        return payloads


@register
class PostEmbeddingBackfillStreamTaskGenerator(StreamTaskGenerator):
    TASK_GENERATOR_TYPE = POST_EMBEDDING_BACKFILL_STREAM
    PLANS_TO_INJECT = {PlanPostEmbeddingV8SearchBackfillBatch.KEY}

    def _get_loader(self):
        return KafkaMmBackfillRequestLoader(TOPIC_BACKFILL)


@register
class PostEmbeddingV5ForReplyStreamTaskGenerator(StreamTaskGenerator):
    TASK_GENERATOR_TYPE = POST_EMBEDDING_V5_FOR_REPLY_STREAM
    PLANS_TO_INJECT = {PlanPostEmbeddingV5ForReply.KEY}

    def _get_loader(self):
        return KafkaPostLoader(TOPIC_MIN_TRACTION_MULTI_MODAL)


@register
class PostEmbeddingV82StreamTaskGenerator(StreamTaskGenerator):
    TASK_GENERATOR_TYPE = POST_EMBEDDING_V8_2_STREAM
    PLANS_TO_INJECT = {PlanPostEmbeddingV82.KEY}

    def _get_loader(self):
        return KafkaPostLoader(TOPIC_REQUESTS_WITH_SUMMARY)


@register
class PostEmbeddingV82RecoveryStreamTaskGenerator(StreamTaskGenerator):
    TASK_GENERATOR_TYPE = POST_EMBEDDING_V8_2_RECOVERY_STREAM
    PLANS_TO_INJECT = {PlanPostEmbeddingV82.KEY}

    def _get_loader(self):
        return KafkaPostLoader(TOPIC_V8_RECOVERY)


@register
class PostEmbeddingV82ForReplyStreamTaskGenerator(StreamTaskGenerator):
    TASK_GENERATOR_TYPE = POST_EMBEDDING_V8_2_FOR_REPLY_STREAM
    PLANS_TO_INJECT = {PlanPostEmbeddingV82ForReply.KEY}

    def _get_loader(self):
        return KafkaPostLoader(TOPIC_MIN_TRACTION_MULTI_MODAL)


@register
class PostEmbeddingV8SearchStreamTaskGenerator(StreamTaskGenerator):
    TASK_GENERATOR_TYPE = POST_EMBEDDING_V8_SEARCH_STREAM
    PLANS_TO_INJECT = {PlanPostEmbeddingV8Search.KEY}

    def _get_loader(self):
        return KafkaPostLoader(TOPIC_REQUESTS_WITH_SUMMARY)


@register
class PostEmbeddingV8SearchForReplyStreamTaskGenerator(StreamTaskGenerator):
    TASK_GENERATOR_TYPE = POST_EMBEDDING_V8_SEARCH_FOR_REPLY_STREAM
    PLANS_TO_INJECT = {PlanPostEmbeddingV8SearchForReply.KEY}

    def _get_loader(self):
        return KafkaPostLoader(TOPIC_MIN_TRACTION_MULTI_MODAL)
