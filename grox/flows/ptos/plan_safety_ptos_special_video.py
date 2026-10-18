from grox.core.plans.plan import Plan
from grox.core.registry import register
from grox.core.tasks.task_media import TaskMediaHydration
from grox.flows.ptos.task_special_video_screen import TaskSpecialVideoScreen
from grox.flows.ptos.task_safety_ptos_adult_content_cross_validation import (
    TaskSafetyPtosAdultContentCrossValidation,
)
from grox.flows.ptos.task_rate_limit import TaskRateLimitSafetyPtosAnnotationWithPost
from grox.flows.ptos.task_safety_ptos_category import TaskSafetyPtosCategoryDetection
from grox.flows.ptos.task_safety_ptos_policy import TaskSafetyPtosPolicyDetection
from grox.flows.ptos.task_safety_ptos_safemodel_sex_nudity import (
    TaskSafetyPtosSafemodelSexNudity,
)
from grox.flows.ptos.task_safety_ptos_special_video_filter import (
    TaskSafetyPtosSpecialVideoFilter,
)
from grox.flows.ptos.task_write_safety_post_annotations_result_sink import (
    TaskWriteSafetyPostAnnotationsResultSink,
)


@register
class PlanSafetyPtosSpecialVideo(Plan):
    KEY = "safety_ptos_special_video"

    TASKS = {
        "task_safety_ptos_special_video_filter": TaskSafetyPtosSpecialVideoFilter,
        "task_safety_ptos_annotation_rate_limit": TaskRateLimitSafetyPtosAnnotationWithPost,
        "task_media_hydration": TaskMediaHydration,
        "task_special_video_screen": TaskSpecialVideoScreen,
        "task_safety_ptos_category_detection": TaskSafetyPtosCategoryDetection,
        "task_safety_ptos_policy_detection": TaskSafetyPtosPolicyDetection,
        "task_safety_ptos_safemodel_sex_nudity": TaskSafetyPtosSafemodelSexNudity,
        "task_safety_ptos_adult_content_cross_validation": TaskSafetyPtosAdultContentCrossValidation,
        "task_write_safety_post_annotations_result_sink": TaskWriteSafetyPostAnnotationsResultSink,
    }

    TASK_DEPENDENCIES = {
        "task_safety_ptos_special_video_filter": {},
        "task_safety_ptos_annotation_rate_limit": {
            "task_safety_ptos_special_video_filter"
        },
        "task_media_hydration": {"task_safety_ptos_annotation_rate_limit"},
        "task_special_video_screen": {"task_media_hydration"},
        "task_safety_ptos_category_detection": {"task_special_video_screen"},
        "task_safety_ptos_policy_detection": {"task_safety_ptos_category_detection"},
        "task_safety_ptos_safemodel_sex_nudity": {"task_safety_ptos_policy_detection"},
        "task_safety_ptos_adult_content_cross_validation": {
            "task_safety_ptos_policy_detection",
            "task_safety_ptos_safemodel_sex_nudity",
        },
        "task_write_safety_post_annotations_result_sink": {
            "task_safety_ptos_adult_content_cross_validation"
        },
    }
