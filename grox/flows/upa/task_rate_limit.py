from cachetools import TTLCache

from grox.core.tasks.task_rate_limit import TaskTTLDedupeWithPost


class TaskRateLimitBangerAnnotationWithPost(TaskTTLDedupeWithPost):
    DEDUPE_CACHE = TTLCache(maxsize=10_000, ttl=60)
    DEDUPE_NAME = "banger"


class TaskRateLimitPostSafetyAnnotationWithPost(TaskTTLDedupeWithPost):
    DEDUPE_CACHE = TTLCache(maxsize=10_000, ttl=60)
    DEDUPE_NAME = "post safety"
