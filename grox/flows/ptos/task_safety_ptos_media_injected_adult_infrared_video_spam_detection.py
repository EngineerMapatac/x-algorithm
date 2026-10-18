import asyncio
import logging
import re
import traceback
import uuid

from grox.config.config import ModelName, grox_config
from grox.core.data_loaders.data_types import Post
from grox.core.data_loaders.media_reference_bundle import (
    MediaReferenceBundle,
    MediaReferenceBundleLoader,
)
from grox.core.lm.convo import (
    THINKING_CONTROL_END,
    THINKING_CONTROL_START,
    Conversation,
    Message,
    Role,
)
from grox.core.lm.post import PostRenderer
from grox.core.schedules.types import TaskContext
from grox.core.tasks.task import Task, TaskResultCategory, TaskWithPost
from grox.flows.ptos.classifier import _strip_thinking_restrictions
from grox.flows.ptos.constants import MEDIA_INJECTED_REASONING_FAV
from grox.flows.ptos.prompts import (
    media_injected_adult_infrared_video_spam_detection_prompt,
)
from grox.flows.ptos.state import (
    SafetyPolicy,
    SafetyPolicyCategory,
    SafetyPolicyType,
    SafetyPostAnnotations,
    SafetyPtosState,
    SafetyPtosViolatedPolicy,
)
from grox.flows.ptos.task_safety_ptos_media_injected_adult_infrared_video_spam_detection_filter import (
    post_videos,
)
from grok_sampler.config import GrokModelConfig
from grok_sampler.vision_sampler import VisionSampler
from monitor.metrics import Metrics
from pydantic import BaseModel

logger = logging.getLogger(__name__)

FLOW_NAME = "media_injected_adult_infrared_video_spam_detection"
MEDIA_REFERENCE_BUNDLE = "media_injected_adult_infrared_video_spam"
_TASK_TIMEOUT_S = 60.0
_METRIC_PREFIX = f"task.safety_ptos_{FLOW_NAME}"
_RESULT_PATTERN = re.compile(r"(.*)<json>(.*)</json>", re.DOTALL)


class MediaInjectedAdultInfraredVideoSpamVerdict(BaseModel):
    violation: bool
    reason: str | None = None


class UnparseableVerdict(ValueError):
    pass


class TaskSafetyPtosMediaInjectedAdultInfraredVideoSpamDetection(TaskWithPost):
    _llm: VisionSampler | None = None

    @classmethod
    def llm(cls) -> VisionSampler:
        if cls._llm is None:
            vlm_config = grox_config.get_model(
                ModelName.GROK_4_1_FAST_HEDGEHOG_CRITICAL
            )
            cls._llm = VisionSampler(GrokModelConfig(**vlm_config.model_dump()))
        return cls._llm

    @classmethod
    async def _exec_with_post(cls, ctx: TaskContext, post: Post) -> None:
        bundle = await MediaReferenceBundleLoader.current(MEDIA_REFERENCE_BUNDLE)
        if not bundle or not bundle.videos:
            raise RuntimeError(f"{FLOW_NAME}: no media reference bundle loaded")
        try:
            await asyncio.wait_for(cls._run(ctx, post, bundle), timeout=_TASK_TIMEOUT_S)
        except asyncio.TimeoutError:
            Metrics.counter(f"{_METRIC_PREFIX}.error.count").add(
                1, attributes={"reason": "timeout"}
            )
            logger.warning(
                f"Post {post.id}: {FLOW_NAME} timed out after {_TASK_TIMEOUT_S}s"
            )
        except UnparseableVerdict as e:
            Metrics.counter(f"{_METRIC_PREFIX}.error.count").add(
                1, attributes={"reason": "unparseable"}
            )
            logger.warning(f"Post {post.id}: {FLOW_NAME} unparseable verdict: {e}")
        except Exception:
            Metrics.counter(f"{_METRIC_PREFIX}.error.count").add(
                1, attributes={"reason": "exception"}
            )
            logger.error(
                f"Post {post.id}: {FLOW_NAME} failed: {traceback.format_exc()}"
            )

    @classmethod
    async def _run(
        cls, ctx: TaskContext, post: Post, bundle: MediaReferenceBundle
    ) -> None:
        if not any(v.convo_video and v.convo_video.frames for v in post_videos(post)):
            Metrics.counter(f"{_METRIC_PREFIX}.skipped.count").add(
                1, attributes={"reason": "no_hydrated_video"}
            )
            return

        use_reasoning = post.get_fav_count() >= MEDIA_INJECTED_REASONING_FAV
        Metrics.counter(f"{_METRIC_PREFIX}.invoked.count").add(1)
        convo = cls.build_convo(post, bundle, use_reasoning)
        raw = await cls.llm().sample(
            convo.interleave(), conversation_id=convo.conversation_id
        )
        logger.info(
            f"{FLOW_NAME} result for post {post.id} use_reasoning={use_reasoning} bundle={bundle.version}/{bundle.digest} conversation_id={convo.conversation_id}: {raw}"
        )

        verdict = cls.parse_verdict(raw)
        Metrics.counter(f"{_METRIC_PREFIX}.verdict.count").add(
            1, attributes={"violation": str(verdict.violation).lower()}
        )
        if verdict.violation:
            logger.info(
                f"{FLOW_NAME} FLAGGED post {post.id} favs={post.get_fav_count()} bundle={bundle.version}/{bundle.digest}: {verdict.reason}"
            )
        cls.apply_verdict(ctx.state(SafetyPtosState), verdict)

    @classmethod
    def build_convo(
        cls, post: Post, bundle: MediaReferenceBundle, use_reasoning: bool = False
    ) -> Conversation:
        convo = Conversation(conversation_id=uuid.uuid4().hex)
        prompt = media_injected_adult_infrared_video_spam_detection_prompt()
        system_prompt = (
            _strip_thinking_restrictions(prompt) if use_reasoning else prompt
        )
        convo.messages.append(Message(role=Role.SYSTEM, content=[system_prompt]))

        user_msg = Message(role=Role.USER, content=[])
        user_msg.content.append(
            f"Reference storyboards from {len(bundle.videos)} confirmed violating videos (frames in chronological order):\n"
        )
        for video_idx, video in enumerate(bundle.videos):
            user_msg.content.extend([f" [Reference video {video_idx + 1}] ", video])
        user_msg.content.append("\n\nPost under review:")
        user_msg.content.extend(
            PostRenderer.render(post.model_copy(update={"descendants": None}))
        )
        user_msg.content.append(
            f"\n\nCompare the video storyboard(s) of post {post.id} against the reference storyboards and provide the requested JSON object.{THINKING_CONTROL_START}"
        )
        convo.messages.append(user_msg)
        assistant_content = [] if use_reasoning else [THINKING_CONTROL_END]
        convo.messages.append(
            Message(role=Role.ASSISTANT, content=assistant_content, separator="")
        )
        return convo

    @staticmethod
    def parse_verdict(raw: str) -> MediaInjectedAdultInfraredVideoSpamVerdict:
        match = _RESULT_PATTERN.search(raw)
        if not match:
            raise UnparseableVerdict(f"no <json> block in output: {raw[-300:]!r}")
        try:
            return MediaInjectedAdultInfraredVideoSpamVerdict.model_validate_json(
                match.group(2).strip()
            )
        except Exception as e:
            raise UnparseableVerdict(f"invalid verdict json: {e}") from e

    @staticmethod
    def apply_verdict(
        state: SafetyPtosState, verdict: MediaInjectedAdultInfraredVideoSpamVerdict
    ) -> None:
        if not verdict.violation:
            state.annotations = SafetyPostAnnotations(violatedPolicies=[])
            return
        reason = f"{FLOW_NAME}: {verdict.reason or 'disguised sexual video matching reference frames'}"
        violation = SafetyPtosViolatedPolicy(
            category=SafetyPolicyCategory.Spam,
            reason=reason,
            safetyPolicy=SafetyPolicy(
                policyType=SafetyPolicyType.SpamManipulatedMedia, reason=reason
            ),
        )
        state.annotations = SafetyPostAnnotations(violatedPolicies=[violation])

    @classmethod
    async def exec(cls, ctx: TaskContext) -> TaskResultCategory:
        return await Task.exec.__wrapped__(cls, ctx)
