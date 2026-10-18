from typing import override

from monitor.metrics import Metrics

from grox.core.data_loaders.data_types import Post
from grox.core.schedules.types import TaskContext
from grox.core.tasks.task_filters import TaskFilterWithPost


class TaskPostEmbeddingForReplyFilter(TaskFilterWithPost):
    @override
    @classmethod
    async def _eligible_with_post(cls, post: Post, ctx: TaskContext) -> bool:
        if not post.ancestors:
            Metrics.counter("task.filter.skipped.count").add(
                1,
                attributes={
                    "filter": "post_embedding_for_reply",
                    "reason": "is_original",
                },
            )
            return False
        if not post.user:
            Metrics.counter("task.filter.skipped.count").add(
                1,
                attributes={"filter": "post_embedding_for_reply", "reason": "no_user"},
            )
            return False
        if not post.ancestors[-1].user or not post.ancestors[0].user:
            Metrics.counter("task.filter.skipped.count").add(
                1,
                attributes={
                    "filter": "post_embedding_for_reply",
                    "reason": "ancestor_no_user",
                },
            )
            return False
        in_reply_user_protected = post.ancestors[-1].user.is_protected
        root_user_protected = post.ancestors[0].user.is_protected
        if in_reply_user_protected or root_user_protected or post.user.is_protected:
            Metrics.counter("task.filter.skipped.count").add(
                1,
                attributes={
                    "filter": "post_embedding_for_reply",
                    "reason": "private_account",
                },
            )
            return False
        Metrics.counter("task.post_embedding_for_reply.eligible.count").add(1)
        return True
