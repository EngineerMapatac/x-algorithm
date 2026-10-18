from grox.core.plans.plan import Plan
from grox.core.registry import register
from grox.core.tasks.task_media import TaskMediaHydration
from grox.flows.reply_spam.task_multi_step_reply_spam import (
    TaskMultiStepReplySpamDetection,
    TaskMultiStepReplySpamFilter,
    TaskWriteMultiStepReplySpamReplyRanking,
)


@register
class PlanMultiStepReplySpam(Plan):
    KEY = "multi_step_reply_spam"

    TASKS = {
        "task_multi_step_reply_spam_filter": TaskMultiStepReplySpamFilter,
        "task_media_hydration": TaskMediaHydration,
        "task_multi_step_reply_spam_detection": TaskMultiStepReplySpamDetection,
        "task_write_multi_step_reply_spam_reply_ranking": TaskWriteMultiStepReplySpamReplyRanking,
    }

    TASK_DEPENDENCIES = {
        "task_multi_step_reply_spam_filter": set(),
        "task_media_hydration": {"task_multi_step_reply_spam_filter"},
        "task_multi_step_reply_spam_detection": {"task_media_hydration"},
        "task_write_multi_step_reply_spam_reply_ranking": {
            "task_multi_step_reply_spam_detection"
        },
    }
