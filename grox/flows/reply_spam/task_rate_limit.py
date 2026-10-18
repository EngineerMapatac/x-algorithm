from cachetools import TTLCache

from grox.core.tasks.task_rate_limit import TaskTTLDedupeWithPost


class TaskRateLimitCoordinatedSpamAnnotationWithPost(TaskTTLDedupeWithPost):
    DEDUPE_CACHE = TTLCache(maxsize=10_000, ttl=60)
    DEDUPE_NAME = "coordinated spam"
