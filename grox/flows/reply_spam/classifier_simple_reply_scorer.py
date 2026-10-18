import logging
import re
import uuid

import json_repair
from grok_sampler.oai_sampler import OaiSampler
from grox.config.config import grox_config
from grox.core.data_loaders.data_types import Post
from grox.core.lm.convo import Conversation, Message, Role
from grox.core.lm.thread import ThreadRenderer
from grox.flows.reply_spam.classifier_reply_ranking import ReplyScorer
from grox.flows.reply_spam.constants import (
    GEMMA_2,
    GEMMA_REPLY_SPAM,
    GEMMA_REPLY_SPAM_MIN_FOLLOWERS,
)
from grox.flows.reply_spam.prompts import reply_scoring_system_simple_prompt
from grox.flows.reply_spam.state_reply_ranking import ReplyScoreResult
from monitor.metrics import Metrics
from pydantic import ValidationError

logger = logging.getLogger(__name__)


class SimpleReplyScorer:
    def __init__(self):
        self.oai_gemma4 = OaiSampler(grox_config.get_oai_model(GEMMA_2))
        self.oai_gemma4_reply_spam = OaiSampler(
            grox_config.get_oai_model(GEMMA_REPLY_SPAM)
        )
        self.full_scorer = ReplyScorer()

    async def score(self, post: Post) -> ReplyScoreResult:
        if post.quoted_post is not None:
            Metrics.counter("task.spam_comment_detection.full_scorer.count").add(1)
            return await self.full_scorer.score(post)
        convo = await self._to_convo(post)
        result = await self._sample(convo, post)
        parsed = await self._parse(result)
        Metrics.histogram(
            "ranked_replies_scores",
            explicit_bucket_boundaries_advisory=[0.0, 1.0, 2.0, 3.0],
        ).record(parsed.score)
        return parsed

    async def _to_convo(self, post: Post) -> Conversation:
        convo = Conversation(conversation_id=uuid.uuid4().hex)
        system_prompt = reply_scoring_system_simple_prompt(100_000)
        convo.messages.append(Message(role=Role.SYSTEM, content=[system_prompt]))
        convo.messages.append(
            ThreadRenderer.render(
                post, role=Role.HUMAN, include_signals=True, include_follower_count=True
            )
        )
        return convo

    def _sampler(self, post: Post) -> OaiSampler:
        thread_followers = max(
            (p.user.follower_count or 0)
            for p in (post.ancestors[0], post.ancestors[-1])
            if p.user
        )
        return (
            self.oai_gemma4_reply_spam
            if thread_followers > GEMMA_REPLY_SPAM_MIN_FOLLOWERS
            else self.oai_gemma4
        )

    async def _sample(self, convo: Conversation, post: Post) -> str:
        return await self._sampler(post).sample(
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

    async def _parse(self, output: str) -> ReplyScoreResult:
        score = None
        reason = ""

        match = re.search(r"\{.*\}", output, re.DOTALL)
        if match and "score" in match.group(0):
            raw_result = match.group(0).strip()
        else:
            raw_result = output

        cleaned_result = await self._clean_output(raw_result)

        try:
            result = ReplyScoreResult.model_validate_json(cleaned_result)
            score = result.score
            reason = result.reason
        except (ValidationError, ValueError):
            try:
                repaired = json_repair.repair_json(cleaned_result, return_objects=True)
                if isinstance(repaired, dict) and "score" in repaired:
                    result = ReplyScoreResult.model_validate(repaired)
                    score = result.score
                    reason = result.reason
                    Metrics.counter("task.reply_ranker.json_repaired.count").add(1)
            except Exception:
                pass

            if score is None:
                score_match = re.search(r'"score":\s*([\d.]+)', cleaned_result)
                if score_match:
                    try:
                        score = float(score_match.group(1).strip())
                    except ValueError:
                        score = None
                        Metrics.counter("task.reply_ranker.invalid.count").add(
                            1,
                            attributes={
                                "filter": "reply_ranking",
                                "reason": "invalid_score_format",
                            },
                        )
            if not reason:
                reason_match = re.search(
                    r'"reason":\s*"((?:[^"\\]|\\.)*)"', cleaned_result, re.DOTALL
                )
                if reason_match:
                    reason = reason_match.group(1)

        if not score and score != 0:
            logger.error(f"Invalid output format: {output}")
            Metrics.counter("task.reply_ranker.invalid.count").add(
                1, attributes={"filter": "reply_ranking", "reason": "invalid_format"}
            )
            raise ValueError(f"Invalid output: {output}")

        if not (0 <= score <= 3):
            logger.error(f"Score {score} outside the 0-3 rubric: {output}")
            Metrics.counter("task.reply_ranker.invalid.count").add(
                1,
                attributes={"filter": "reply_ranking", "reason": "score_out_of_range"},
            )
            raise ValueError(f"Score {score} outside the 0-3 rubric")

        return ReplyScoreResult(score=score, reason=reason)
