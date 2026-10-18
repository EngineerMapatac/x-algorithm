import asyncio
import logging
from thrifts.gen.twitter.strato.columns.content_understanding.content_understanding.ttypes import (
    SimpleTweetEmbedding,
)
from thrifts.serdes import Serializer
from grox.core.data_loaders.data_types import Post
from grox.config.config import grox_config
from kafka_cli.multi_region_producer import MultiRegionKafkaProducer
from grox.flows.mm_emb.constants import (
    TOPIC_EMBEDDING_V5,
    TOPIC_EMBEDDING_V5_ALL,
    TOPIC_EMBEDDING_V8_2,
    TOPIC_EMBEDDING_V8_SEARCH,
)

logger = logging.getLogger(__name__)


def _serialize(post: Post, embedding: list[float]) -> bytes:
    return Serializer.serialize(
        SimpleTweetEmbedding(tweetId=int(post.id), embedding1=embedding)
    )


class TaskPublishEmbeddingMultiRegionKafka:
    KAFKA_TOPIC_NAME: str

    _producer: MultiRegionKafkaProducer | None = None
    _producer_lock = asyncio.Lock()

    @classmethod
    async def _publish_to_kafka(cls, post: Post, embedding: list[float]) -> None:
        producer = await cls._get_kafka_producer()
        await producer.send(id=post.id, value=_serialize(post, embedding))
        logger.info(f"Published embedding for post {post.id} to {cls.KAFKA_TOPIC_NAME}")

    @classmethod
    async def _get_kafka_producer(cls) -> MultiRegionKafkaProducer:
        if cls._producer is None:
            async with cls._producer_lock:
                if cls._producer is None:
                    producer = MultiRegionKafkaProducer(
                        grox_config.get_kafka_producer(cls.KAFKA_TOPIC_NAME)
                    )
                    await producer.start()
                    cls._producer = producer
        return cls._producer


class TaskPublishEmbeddingV5Kafka(TaskPublishEmbeddingMultiRegionKafka):
    KAFKA_TOPIC_NAME = TOPIC_EMBEDDING_V5


class TaskPublishEmbeddingV5AllKafka(TaskPublishEmbeddingMultiRegionKafka):
    KAFKA_TOPIC_NAME = TOPIC_EMBEDDING_V5_ALL


class TaskPublishEmbeddingV82Kafka(TaskPublishEmbeddingMultiRegionKafka):
    KAFKA_TOPIC_NAME = TOPIC_EMBEDDING_V8_2


class TaskPublishEmbeddingV8SearchKafka(TaskPublishEmbeddingMultiRegionKafka):
    KAFKA_TOPIC_NAME = TOPIC_EMBEDDING_V8_SEARCH
