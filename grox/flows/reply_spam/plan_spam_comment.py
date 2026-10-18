from grox.core.plans.plan import Plan
from grox.core.registry import register
from grox.flows.reply_spam.task_write import TaskWriteReplyRankingManhattan
from grox.core.tasks.task_media import TaskMediaHydration
from grox.flows.reply_spam.task_filter import TaskSpamFilter
from grox.flows.reply_spam.task_spam_detection import TaskSpamDetection


@register
class PlanSpamComment(Plan):
    KEY = "spam_comment"

    TASKS = {
        "task_spam_filter": TaskSpamFilter,
        "task_media_hydration": TaskMediaHydration,
        "task_spam_detection": TaskSpamDetection,
        "task_write_reply_ranking_manhattan": TaskWriteReplyRankingManhattan,
    }

    TASK_DEPENDENCIES = {
        "task_spam_filter": set(),
        "task_media_hydration": {"task_spam_filter"},
        "task_spam_detection": {"task_media_hydration"},
        "task_write_reply_ranking_manhattan": {"task_spam_detection"},
    }
