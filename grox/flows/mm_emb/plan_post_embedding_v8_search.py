from grox.core.plans.plan import Plan
from grox.core.registry import register
from grox.core.tasks.task_asr import TaskASRTranscription
from grox.core.tasks.task_media import TaskMediaHydration
from grox.flows.mm_emb.task_multimodal_post_embedding import (
    TaskMultimodalPostEmbeddingV8Search,
)
from grox.flows.mm_emb.task_rate_limit import TaskRateLimitEmbeddingV8Search
from grox.flows.mm_emb.task_write_mm_embedding_sink import (
    TaskWriteV8SearchSinkSkipKafkaForReplies,
)


@register
class PlanPostEmbeddingV8Search(Plan):
    KEY = "mm_emb_v8_search"

    TASKS = {
        "task_post_embedding_rate_limit_v8_search": TaskRateLimitEmbeddingV8Search,
        "task_media_hydration": TaskMediaHydration,
        "task_asr_transcription": TaskASRTranscription,
        "task_multimodal_post_embedding_v8_search": TaskMultimodalPostEmbeddingV8Search,
        "task_write_post_embedding_sink_v8_search": TaskWriteV8SearchSinkSkipKafkaForReplies,
    }

    TASK_DEPENDENCIES = {
        "task_post_embedding_rate_limit_v8_search": set(),
        "task_media_hydration": {"task_post_embedding_rate_limit_v8_search"},
        "task_asr_transcription": {"task_media_hydration"},
        "task_multimodal_post_embedding_v8_search": {"task_asr_transcription"},
        "task_write_post_embedding_sink_v8_search": {
            "task_multimodal_post_embedding_v8_search"
        },
    }
