from grox.core.plans.plan import Plan
from grox.core.registry import register
from grox.core.tasks.task_asr import TaskASRTranscription
from grox.core.tasks.task_load_post import TaskLoadPost
from grox.core.tasks.task_media import TaskMediaHydration
from grox.flows.mm_emb.task_backfill_gate import (
    TaskBackfillFavGate,
    TaskBackfillPreloadGate,
)
from grox.flows.mm_emb.task_multimodal_post_embedding import (
    TaskMultimodalPostEmbeddingV8SearchBackfill,
)
from grox.flows.mm_emb.task_rate_limit import TaskRateLimitEmbeddingV8SearchBackfill
from grox.flows.mm_emb.task_write_mm_embedding_sink import TaskWriteV8SearchBackfillSink


@register
class PlanPostEmbeddingV8SearchBackfill(Plan):
    KEY = "mm_emb_v8_search_backfill"

    TASKS = {
        "task_backfill_fav_gate": TaskBackfillFavGate,
        "task_backfill_preload_gate": TaskBackfillPreloadGate,
        "task_load_post": TaskLoadPost,
        "task_post_embedding_rate_limit": TaskRateLimitEmbeddingV8SearchBackfill,
        "task_media_hydration": TaskMediaHydration,
        "task_asr_transcription": TaskASRTranscription,
        "task_multimodal_post_embedding_v8_search": TaskMultimodalPostEmbeddingV8SearchBackfill,
        "task_write_post_embedding_sink_v8_search": TaskWriteV8SearchBackfillSink,
    }

    TASK_DEPENDENCIES = {
        "task_backfill_fav_gate": set(),
        "task_backfill_preload_gate": {"task_backfill_fav_gate"},
        "task_load_post": {"task_backfill_preload_gate"},
        "task_post_embedding_rate_limit": {"task_load_post"},
        "task_media_hydration": {"task_post_embedding_rate_limit"},
        "task_asr_transcription": {"task_media_hydration"},
        "task_multimodal_post_embedding_v8_search": {"task_asr_transcription"},
        "task_write_post_embedding_sink_v8_search": {
            "task_multimodal_post_embedding_v8_search"
        },
    }
