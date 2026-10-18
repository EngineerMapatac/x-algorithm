import asyncio
import logging
from typing import override


from grox.core.constants import GORK_USER_ID, GROK_USER_ID
from grox.core.data_loaders.data_types import Post
from grox.core.data_loaders.strato_loader import UserStratoLoader
from grox.core.schedules.types import TaskContext
from grox.core.tasks.disable_rules import DisableTaskForNonProd
from grox.core.tasks.task import Task, TaskResultCategory, TaskWithPost
from grox.core.tasks.task_filters import TaskFilterWithPost
from grox.flows.reply_spam.classifier_multi_step_reply_spam import (
    MultiStepReplySpamScorer,
)
from grox.flows.reply_spam.state_multi_step_reply_spam import MultiStepReplySpamState
from grox.flows.reply_spam.strato_loader import ReplyRankingScoreStratoLoader
from grox.flows.reply_spam.task_write import _apply_reply_spam_label
from monitor.metrics import Metrics
from strato_http.queries.data_types import ReplyRankingScore

logger = logging.getLogger(__name__)


class TaskMultiStepReplySpamFilter(TaskFilterWithPost):
    FOLLOWER_COUNT_THRESHOLD_FOR_SPAM_DETECTION = 1000
    FILTER_NAME = "multi_step_reply_spam"

    @override
    @classmethod
    async def _eligible_with_post(cls, post: Post, ctx: TaskContext) -> bool:
        if not post.ancestors:
            Metrics.counter("task.filter.skipped.count").add(
                1, attributes={"filter": cls.FILTER_NAME, "reason": "not_reply"}
            )
            return False
        if any(a is None for a in post.ancestors):
            Metrics.counter("task.filter.skipped.count").add(
                1, attributes={"filter": cls.FILTER_NAME, "reason": "deleted_ancestor"}
            )
            return False
        if not post.user:
            Metrics.counter("task.filter.skipped.count").add(
                1, attributes={"filter": cls.FILTER_NAME, "reason": "no_user"}
            )
            return False
        if post.user.id == GROK_USER_ID:
            Metrics.counter("task.filter.skipped.count").add(
                1, attributes={"filter": cls.FILTER_NAME, "reason": "is_grok_reply"}
            )
            return False
        if post.user.id == GORK_USER_ID:
            Metrics.counter("task.filter.skipped.count").add(
                1, attributes={"filter": cls.FILTER_NAME, "reason": "is_gork_reply"}
            )
            return False
        if not post.ancestors[-1].user:
            Metrics.counter("task.filter.skipped.count").add(
                1,
                attributes={
                    "filter": cls.FILTER_NAME,
                    "reason": "previous_post_no_user",
                },
            )
            return False
        if not post.ancestors[0].user:
            Metrics.counter("task.filter.skipped.count").add(
                1, attributes={"filter": cls.FILTER_NAME, "reason": "root_post_no_user"}
            )
            return False
        if post.user.id == post.ancestors[0].user.id:
            logger.info(
                f"Skipping multi-step reply spam since the replier is same as reply root post {post.id}"
            )
            Metrics.counter("task.filter.skipped.count").add(
                1,
                attributes={
                    "filter": cls.FILTER_NAME,
                    "reason": "same_user_reply_as_root",
                },
            )
            return False
        if len(post.ancestors) < 2:
            logger.info(
                f"Skipping multi-step reply spam since the reply thread is not more than two level deep {post.id}"
            )
            Metrics.counter("task.filter.skipped.count").add(
                1, attributes={"filter": cls.FILTER_NAME, "reason": "one_level_deep"}
            )
            return False
        if post.ancestors[-1].user.id != post.user.id:
            Metrics.counter("task.filter.skipped.count").add(
                1,
                attributes={
                    "filter": cls.FILTER_NAME,
                    "reason": "parent_not_same_author",
                },
            )
            return False
        root_user_follower_count = post.ancestors[0].user.follower_count or 0
        if root_user_follower_count < cls.FOLLOWER_COUNT_THRESHOLD_FOR_SPAM_DETECTION:
            Metrics.counter("task.filter.skipped.count").add(
                1, attributes={"filter": cls.FILTER_NAME, "reason": "low_blast_radius"}
            )
            return False
        is_high_page_rank, is_grey_badge = await asyncio.gather(
            UserStratoLoader.is_high_page_rank_v2_user(post.user.id),
            UserStratoLoader.is_grey_badge_user(post.user.id),
        )
        if is_high_page_rank or is_grey_badge:
            logger.info(
                f"Skipping multi-step reply spam for post {post.id} user {post.user.id} "
                f"(high_page_rank_v2={is_high_page_rank}, grey_badge={is_grey_badge})"
            )
            Metrics.counter("task.filter.skipped.count").add(
                1,
                attributes={
                    "filter": cls.FILTER_NAME,
                    "reason": "high_page_rank_or_grey_badge",
                },
            )
            return False
        Metrics.counter("task.multi_step_reply_spam_filter.eligible.count").add(1)
        return True


class TaskMultiStepReplySpamDetection(TaskWithPost):
    scorer = MultiStepReplySpamScorer()

    @classmethod
    async def exec(cls, ctx: TaskContext) -> TaskResultCategory:
        return await Task.exec.__wrapped__(cls, ctx)

    @classmethod
    async def _exec_with_post(cls, ctx: TaskContext, post: Post) -> None:
        result = await cls.scorer.score(post)
        ctx.state(MultiStepReplySpamState).result = result
        if result.spam_post_ids:
            logger.info(
                f"Multi-step reply spam found for post {post.id}: spam_post_ids={result.spam_post_ids} reason={result.reason!r}"
            )
            Metrics.counter("task.multi_step_reply_spam_detection.positive.count").add(
                1
            )
        else:
            Metrics.counter("task.multi_step_reply_spam_detection.negative.count").add(
                1
            )


class TaskWriteMultiStepReplySpamReplyRanking(Task):
    DISABLE_RULES = [DisableTaskForNonProd]

    @classmethod
    async def _exec(cls, ctx: TaskContext) -> None:
        post = ctx.payload.post
        if not post:
            return
        result = ctx.state(MultiStepReplySpamState).result
        if result is None:
            Metrics.counter("task.write_multi_step_reply_spam.skipped.count").add(
                1, attributes={"reason": "no_results"}
            )
            return
        if not result.spam_post_ids:
            Metrics.counter("task.write_multi_step_reply_spam.skipped.count").add(
                1, attributes={"reason": "no_spam"}
            )
            return

        thread_posts = (post.ancestors or []) + [post]
        spam_ids = set(result.spam_post_ids)
        reasoning = (result.reason or "")[-500:]

        Metrics.counter("task.write_multi_step_reply_spam.intaken.count").add(1)
        for p in thread_posts:
            if p.id not in spam_ids:
                continue
            if not p.user:
                Metrics.counter(
                    "task.write_multi_step_reply_spam.skipped_post.count"
                ).add(1, attributes={"reason": "no_author"})
                continue
            await cls._mark_spam(p.id, p.user.id, reasoning)
            Metrics.counter("task.write_multi_step_reply_spam.success.count").add(1)

    @classmethod
    async def _mark_spam(cls, post_id: str, author_id: int, reasoning: str) -> None:
        await _apply_reply_spam_label(post_id, author_id)

        await ReplyRankingScoreStratoLoader.publish_reply_ranking_score(
            post_id=post_id,
            reply_ranking_score=ReplyRankingScore(
                score=0.0, reasoning=reasoning[-500:]
            ),
        )
        logger.info(
            f"Published multi-step reply spam reply ranking score 0 for post {post_id}"
        )
