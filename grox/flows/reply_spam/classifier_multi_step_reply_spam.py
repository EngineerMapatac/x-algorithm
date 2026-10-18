import re
import json
import uuid
import logging

import json_repair

from grok_sampler.oai_sampler import OaiSampler
from grox.core.lm.post import PostRenderer
from grox.core.lm.user import UserRenderer
from grox.core.lm.convo import Role, Message, Conversation
from grox.config.config import grox_config
from grox.flows.reply_spam.prompts import multi_step_reply_spam_system_prompt
from grox.core.data_loaders.data_types import Post
from grox.flows.reply_spam.state_multi_step_reply_spam import MultiStepReplySpamResult
from monitor.metrics import Metrics
from grox.flows.reply_spam.constants import GEMMA_2

logger = logging.getLogger(__name__)

MAX_MEDIA_PER_POST = 2
MAX_THREAD_SIZE = 10


class MultiStepReplySpamScorer:
    model_name = GEMMA_2

    def __init__(self):
        oai_config = grox_config.get_oai_model(GEMMA_2)
        self.oai_gemma4 = OaiSampler(oai_config)

    async def score(self, post: Post) -> MultiStepReplySpamResult:
        convo = await self._to_convo(post)
        output = await self._sample(convo)
        result = await self._parse(post, output)
        result = self._drop_if_newest_reply_not_flagged(post, result)
        result = self._drop_other_author_posts(post, result)
        result = self._drop_singleton(post, result)
        Metrics.counter("multi_step_reply_spam.scored.count").add(
            1, attributes={"is_spam": bool(result.spam_post_ids)}
        )
        return result

    @classmethod
    def _drop_if_newest_reply_not_flagged(
        cls, post: Post, result: MultiStepReplySpamResult
    ) -> MultiStepReplySpamResult:
        if not result.spam_post_ids or post.id in result.spam_post_ids:
            return result
        logger.info(
            f"Dropping multi-step-reply-spam flags for post {post.id}: newest reply not in flagged set "
            f"{result.spam_post_ids} reason={result.reason!r}"
        )
        Metrics.counter("multi_step_reply_spam.newest_not_flagged_dropped.count").add(1)
        return MultiStepReplySpamResult(reason=result.reason)

    @classmethod
    def _drop_other_author_posts(
        cls, post: Post, result: MultiStepReplySpamResult
    ) -> MultiStepReplySpamResult:
        if not result.spam_post_ids or not post.user:
            return result
        author_by_id = {
            p.id: (p.user.id if p.user else None) for p in cls._thread_posts(post)
        }
        kept = [
            pid for pid in result.spam_post_ids if author_by_id.get(pid) == post.user.id
        ]
        if len(kept) == len(result.spam_post_ids):
            return result
        logger.info(
            f"Dropping other-author posts from multi-step-reply-spam flags for post {post.id}: "
            f"{[pid for pid in result.spam_post_ids if pid not in kept]} kept={kept} reason={result.reason!r}"
        )
        Metrics.counter("multi_step_reply_spam.other_author_dropped.count").add(1)
        return MultiStepReplySpamResult(spam_post_ids=kept, reason=result.reason)

    @classmethod
    def _drop_singleton(
        cls, post: Post, result: MultiStepReplySpamResult
    ) -> MultiStepReplySpamResult:
        if len(result.spam_post_ids) != 1:
            return result
        logger.info(
            f"Dropping singleton multi-step-reply-spam flag for post {post.id}: {result.spam_post_ids[0]} reason={result.reason!r}"
        )
        Metrics.counter("multi_step_reply_spam.singleton_dropped.count").add(1)
        return MultiStepReplySpamResult(reason=result.reason)

    @staticmethod
    def _thread_posts(post: Post) -> list[Post]:
        ancestors = post.ancestors or []
        if len(ancestors) > MAX_THREAD_SIZE:
            ancestors = (
                ancestors[: MAX_THREAD_SIZE // 2] + ancestors[-(MAX_THREAD_SIZE // 2) :]
            )
        return ancestors + [post]

    async def _to_convo(self, post: Post) -> Conversation:
        convo = Conversation(conversation_id=uuid.uuid4().hex)
        convo.messages.append(
            Message(role=Role.SYSTEM, content=[multi_step_reply_spam_system_prompt()])
        )

        posts = self._thread_posts(post)
        last = len(posts) - 1
        content: list = []
        content.append(
            "\n# Reply Author Info\n\n"
            "The user profile below describes the author of the reply being evaluated "
            "(the final post of the thread).\n"
        )
        content.extend(UserRenderer.render(post.user))
        content.append(
            "\n\n# Thread\n\nThe reply thread is listed below; each post is labeled with a `#### Post N` index. "
            "Post 0 is the original (root) post and the highest-numbered post is the reply being evaluated. "
            "Refer to posts only by their integer index.\n\n"
        )
        for i, p in enumerate(posts):
            if i == 0:
                tag = " (root post — never flag)"
            elif i == last:
                tag = " (the reply being evaluated)"
            else:
                tag = ""
            content.append(f"\n\n#### Post {i}{tag}\n\n")
            content.extend(
                PostRenderer.render(
                    p,
                    max_media=MAX_MEDIA_PER_POST,
                    include_follower_count=True,
                    include_bio=i == 0,
                )
            )
            content.append("\n\n------\n\n")
        convo.messages.append(Message(role=Role.HUMAN, content=content))
        return convo

    async def _sample(self, convo: Conversation) -> str:
        return await self.oai_gemma4.sample(
            convo.to_openai_messages(), conversation_id=convo.conversation_id
        )

    async def _clean_output(self, output: str) -> str:
        if output.endswith("<|eos|>"):
            output = output.removesuffix("<|eos|>")
        output = output.strip()
        if output.startswith("```json"):
            output = output[7:]
        elif output.startswith("```"):
            output = output[3:]
        if output.endswith("```"):
            output = output[:-3]
        output = output.strip()
        return output

    @staticmethod
    def _resolve_post_ids(posts: list[Post], raw_values: list) -> list[str]:
        id_by_index = {i: p.id for i, p in enumerate(posts)}
        candidate_ids = {p.id for p in posts[1:]}
        out: list[str] = []
        for v in raw_values:
            s = str(v).strip()
            if not s:
                continue
            if s.lstrip("-").isdigit():
                n = int(s)
                if 0 < n < len(posts):
                    pid = id_by_index[n]
                    if pid not in out:
                        out.append(pid)
                    continue
            if s in candidate_ids and s not in out:
                out.append(s)
        return out

    async def _parse(self, post: Post, output: str) -> MultiStepReplySpamResult:
        posts = self._thread_posts(post)
        match = re.search(r"\{.*\}", output, re.DOTALL)
        raw_json = (
            match.group(0).strip()
            if (match and "spam_post" in match.group(0))
            else output
        )
        cleaned = await self._clean_output(raw_json)

        data = None
        try:
            data = json.loads(cleaned)
        except Exception:
            try:
                repaired = json_repair.repair_json(cleaned, return_objects=True)
                if isinstance(repaired, dict):
                    data = repaired
                    Metrics.counter("multi_step_reply_spam.json_repaired.count").add(1)
            except Exception:
                data = None

        if isinstance(data, dict):
            raw_values = data.get("spam_post_indexes")
            if raw_values is None:
                raw_values = data.get("spam_post_ids")
            if raw_values is None:
                raw_values = []
            if not isinstance(raw_values, list):
                raw_values = [raw_values]
            reason = data.get("reason") or ""
            return MultiStepReplySpamResult(
                spam_post_ids=self._resolve_post_ids(posts, raw_values), reason=reason
            )

        arr_match = re.search(
            r'"spam_post_(?:indexes|ids)"\s*:\s*\[([^\]]*)\]', cleaned, re.DOTALL
        )
        if arr_match is None:
            logger.error(f"Invalid multi-step reply spam output format: {output}")
            Metrics.counter("multi_step_reply_spam.invalid.count").add(1)
            raise ValueError(f"Invalid output: {output}")
        raw_values = re.findall(r"\d+", arr_match.group(1))
        reason = ""
        reason_match = re.search(r'"reason":\s*"((?:[^"\\]|\\.)*)"', cleaned, re.DOTALL)
        if reason_match:
            reason = reason_match.group(1)
        return MultiStepReplySpamResult(
            spam_post_ids=self._resolve_post_ids(posts, raw_values), reason=reason
        )
