from grox.core.plans.plan import Plan
from grox.core.registry import register
from grox.flows.mm_emb.task_backfill_fanout import TaskBackfillBatchFanout


@register
class PlanPostEmbeddingV8SearchBackfillBatch(Plan):
    KEY = "mm_emb_v8_search_backfill_batch"

    TASKS = {
        "task_backfill_batch_fanout": TaskBackfillBatchFanout,
    }

    TASK_DEPENDENCIES = {
        "task_backfill_batch_fanout": set(),
    }
