import logging
import traceback
import uuid
from typing import override

from kafka_cli.config import KafkaMessage
from monitor.metrics import Metrics

from grox.core.data_loaders.data_types import Post
from grox.core.data_loaders.kafka_loader import KafkaLoader
from grox.core.data_loaders.message_queue_loader import MessageQueuePayload
from grox.flows.ptos.state import LiveClusterAnchorInput

logger = logging.getLogger(__name__)


class KafkaLiveClusterAnchorLoader(KafkaLoader):
    @override
    def _messages_to_payloads(
        self, messages: list[KafkaMessage]
    ) -> list[MessageQueuePayload]:
        payloads: list[MessageQueuePayload] = []
        for message in messages:
            if message.value is None:
                Metrics.counter("kafka_loader.message_dropped.count").add(
                    1, attributes={"topic": self.topic_name, "reason": "null_value"}
                )
                continue
            try:
                anchor = LiveClusterAnchorInput.from_proto_kafka_message(message.value)
            except Exception:
                Metrics.counter("kafka_loader.message_dropped.count").add(
                    1,
                    attributes={
                        "topic": self.topic_name,
                        "reason": "deserialize_error",
                    },
                )
                logger.error(
                    f"Dropping un-deserializable live_cluster_anchor topic={self.topic_name} error={traceback.format_exc()}"
                )
                continue
            payload = MessageQueuePayload(
                mid=uuid.uuid4().hex,
                post=Post(id=str(anchor.anchorPostId)),
            )
            payload.set_ext(anchor)
            payloads.append(payload)
        return payloads
