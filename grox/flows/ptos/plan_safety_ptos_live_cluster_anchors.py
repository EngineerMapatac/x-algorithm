from grox.core.plans.plan import Plan
from grox.core.registry import register
from grox.core.tasks.task_load_post import TaskLoadPost
from grox.core.tasks.task_media import TaskMediaHydration
from grox.flows.ptos.task_filter import TaskSafetyPtosFilter
from grox.flows.ptos.task_rate_limit import TaskRateLimitSafetyPtosLiveClusterAnchors
from grox.flows.ptos.task_safety_ptos_category import TaskSafetyPtosCategoryDetection
from grox.flows.ptos.task_safety_ptos_policy import TaskSafetyPtosPolicyDetection
from grox.flows.ptos.task_write_live_cluster_anchor_verdict_sink import (
    TaskWriteLiveClusterAnchorVerdictSink,
)


@register
class PlanSafetyPtosLiveClusterAnchors(Plan):
    KEY = "safety_ptos_live_cluster_anchors"

    TASKS = {
        "task_safety_ptos_live_cluster_anchors_rate_limit": TaskRateLimitSafetyPtosLiveClusterAnchors,
        "task_load_post": TaskLoadPost,
        "task_safety_ptos_filter": TaskSafetyPtosFilter,
        "task_media_hydration": TaskMediaHydration,
        "task_safety_ptos_category_detection": TaskSafetyPtosCategoryDetection,
        "task_safety_ptos_policy_detection": TaskSafetyPtosPolicyDetection,
        "task_write_live_cluster_anchor_verdict_sink": TaskWriteLiveClusterAnchorVerdictSink,
    }

    TASK_DEPENDENCIES = {
        "task_safety_ptos_live_cluster_anchors_rate_limit": set(),
        "task_load_post": {"task_safety_ptos_live_cluster_anchors_rate_limit"},
        "task_safety_ptos_filter": {"task_load_post"},
        "task_media_hydration": {"task_safety_ptos_filter"},
        "task_safety_ptos_category_detection": {"task_media_hydration"},
        "task_safety_ptos_policy_detection": {"task_safety_ptos_category_detection"},
        "task_write_live_cluster_anchor_verdict_sink": {
            "task_safety_ptos_policy_detection"
        },
    }
