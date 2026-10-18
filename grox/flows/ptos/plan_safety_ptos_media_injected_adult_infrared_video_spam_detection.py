from grox.core.plans.plan import Plan
from grox.core.registry import register
from grox.core.tasks.task_media import TaskMediaHydration
from grox.flows.ptos.task_rate_limit import (
    TaskRateLimitSafetyPtosMediaInjectedAdultInfraredVideoSpamDetection,
)
from grox.flows.ptos.task_safety_ptos_media_injected_adult_infrared_video_spam_detection import (
    TaskSafetyPtosMediaInjectedAdultInfraredVideoSpamDetection,
)
from grox.flows.ptos.task_safety_ptos_media_injected_adult_infrared_video_spam_detection_filter import (
    TaskSafetyPtosMediaInjectedAdultInfraredVideoSpamDetectionFilter,
)
from grox.flows.ptos.task_write_safety_post_annotations_result_sink import (
    TaskWriteSafetyPostAnnotationsResultSink,
)


@register
class PlanSafetyPtosMediaInjectedAdultInfraredVideoSpamDetection(Plan):
    KEY = "safety_ptos_media_injected_adult_infrared_video_spam_detection"

    TASKS = {
        "task_safety_ptos_annotation_rate_limit": TaskRateLimitSafetyPtosMediaInjectedAdultInfraredVideoSpamDetection,
        "task_safety_ptos_media_injected_adult_infrared_video_spam_detection_filter": TaskSafetyPtosMediaInjectedAdultInfraredVideoSpamDetectionFilter,
        "task_media_hydration": TaskMediaHydration,
        "task_safety_ptos_media_injected_adult_infrared_video_spam_detection": TaskSafetyPtosMediaInjectedAdultInfraredVideoSpamDetection,
        "task_write_safety_post_annotations_result_sink": TaskWriteSafetyPostAnnotationsResultSink,
    }

    TASK_DEPENDENCIES = {
        "task_safety_ptos_annotation_rate_limit": {},
        "task_safety_ptos_media_injected_adult_infrared_video_spam_detection_filter": {
            "task_safety_ptos_annotation_rate_limit"
        },
        "task_media_hydration": {
            "task_safety_ptos_media_injected_adult_infrared_video_spam_detection_filter"
        },
        "task_safety_ptos_media_injected_adult_infrared_video_spam_detection": {
            "task_media_hydration"
        },
        "task_write_safety_post_annotations_result_sink": {
            "task_safety_ptos_media_injected_adult_infrared_video_spam_detection"
        },
    }
