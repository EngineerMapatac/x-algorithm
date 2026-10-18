from __future__ import annotations

import re
from dataclasses import dataclass, replace
from datetime import datetime, timezone
from typing import Any

from grox.config.config import grox_config
from grox.core.data_loaders.data_types import Image, Post, User, Video

IMAGE_PAD = grox_config.prompt_tokens.image_pad

INSTRUCTION_PREFIX = "Represent this X post (formerly Twitter) for recommendation: capture its topic, intent, key entities, and likely engagement."

_MEDIA_URL_RE = re.compile(
    r"\s*https?://(?:twitter|x)\.com/[^/\s]+/status/\d+/(?:photo|video)/\d+\s*$",
    re.IGNORECASE,
)
_MONTHS = (
    "Jan",
    "Feb",
    "Mar",
    "Apr",
    "May",
    "Jun",
    "Jul",
    "Aug",
    "Sep",
    "Oct",
    "Nov",
    "Dec",
)

MAX_MEDIA = 16
MAX_BIO = 500
MAX_POST_TEXT = 4000
MAX_QUOTED_TEXT = 1000
MAX_ASR = 2000
MAX_CARD_TITLE = 100
MAX_ARTICLE_TITLE = 200


@dataclass
class RenderedMedium:
    label: str
    quoted_label: str
    frames: list[bytes]
    transcript: str | None
    is_video: bool
    is_own: bool


def _strip_dangling_media_urls(text: str) -> str:
    return _MEDIA_URL_RE.sub("", text or "").strip()


def _sanitize(text: str) -> str:
    return (text or "").replace(IMAGE_PAD, "").strip()


def _truncate(text: str, limit: int) -> str:
    text = _sanitize(text)
    if limit < 0:
        return text
    return text[:limit].rstrip()


def _humanize_followers(n: int | None) -> str:
    if not n or n <= 0:
        return ""

    def fmt(v: float, s: str) -> str:
        t = f"{v:.1f}"
        return (t[:-2] if t.endswith(".0") else t) + s

    if n >= 1_000_000_000:
        return fmt(n / 1_000_000_000, "B")
    if n >= 1_000_000:
        return fmt(n / 1_000_000, "M")
    if n >= 1_000:
        return fmt(n / 1_000, "K")
    return str(int(n))


_TIER_EDGES = [
    (10, "<10"),
    (100, "10-100"),
    (1_000, "100-1K"),
    (10_000, "1K-10K"),
    (100_000, "10K-100K"),
    (1_000_000, "100K-1M"),
    (10_000_000, "1M-10M"),
    (100_000_000, "10M-100M"),
]


def _tier_label(n: int | None) -> str:
    if n is None or n < 0:
        return ""
    for edge, label in _TIER_EDGES:
        if n < edge:
            return label
    return ">100M"


def _count_with_tier(n: int | None) -> str:
    h = _humanize_followers(n)
    if not h:
        return ""
    t = _tier_label(n)
    return f"{h} ({t})" if t else h


def _format_join_date(msec: int | None) -> str:
    if not msec or msec <= 0:
        return ""
    try:
        dt = datetime.fromtimestamp(msec / 1000.0, tz=timezone.utc)
    except (OverflowError, OSError, ValueError):
        return ""
    return f"{_MONTHS[dt.month - 1]} {dt.year}"


def _humanize_duration(ms: int | None) -> str:
    if not ms or ms <= 0:
        return ""
    secs = int(round(ms / 1000.0))
    m, s = divmod(secs, 60)
    return f"{m}m{s:02d}s" if m else f"{s}s"


def _quality_bucket(bitrate: int | None) -> str:
    if not bitrate or bitrate <= 0:
        return ""
    if bitrate >= 5_000_000:
        return "Full HD"
    if bitrate >= 1_500_000:
        return "HD"
    return "SD"


def _collapse_blank_lines(text: str) -> str:
    out: list[str] = []
    prev_blank = False
    for raw in text.splitlines():
        line = raw.rstrip()
        blank = not line
        if blank and prev_blank:
            continue
        out.append(line)
        prev_blank = blank
    return "\n".join(out).strip()


class V8SearchEmbedPostRenderer:
    @classmethod
    def render_for_embedding(
        cls,
        post: Post,
        core_user: dict[str, Any] | None = None,
        max_frames: int = 4,
    ) -> tuple[str, list[bytes]]:
        images: list[bytes] = []
        blocks: list[str] = [INSTRUCTION_PREFIX, ""]

        blocks.append(cls._author_block(post.user, core_user))
        blocks.append("")

        records = cls._collect_media_records(post, max_frames, is_own=True)
        quoted_post = getattr(post, "quoted_post", None)
        if quoted_post is not None:
            records += cls._collect_media_records(quoted_post, max_frames, is_own=False)
        records = cls._apply_frame_budget(records, MAX_MEDIA)

        if len(records) <= 1:
            rec = records[0] if records else None
            if rec is not None:
                label = rec.label if rec.is_own else rec.quoted_label
                blocks.append(label + IMAGE_PAD * len(rec.frames))
                images.extend(rec.frames)
            if post.full_text:
                blocks.append(
                    f"Post text: {_truncate(_strip_dangling_media_urls(post.full_text), MAX_POST_TEXT)}"
                )
            article = cls._article_block(post)
            if article:
                blocks.append(article)
            card = cls._card_block(post)
            if card:
                blocks.append(card)
            transcript = rec.transcript if (rec is not None and rec.is_video) else None
            if transcript:
                blocks += ["", f"Transcript: {_truncate(transcript, MAX_ASR)}"]
        else:
            transcript_recs = [
                r for r in records if r.is_video and (r.transcript or "").strip()
            ]
            per_video = (
                MAX_ASR
                if len(transcript_recs) <= 1
                else max(1, MAX_ASR // len(transcript_recs))
            )
            for rec in records:
                lbl = rec.label if rec.is_own else rec.quoted_label
                blocks.append(lbl + IMAGE_PAD * len(rec.frames))
                images.extend(rec.frames)
                if rec.is_video and (rec.transcript or "").strip():
                    blocks.append(f"Transcript: {_truncate(rec.transcript, per_video)}")
            if post.full_text:
                blocks.append(
                    f"Post text: {_truncate(_strip_dangling_media_urls(post.full_text), MAX_POST_TEXT)}"
                )
            article = cls._article_block(post)
            if article:
                blocks.append(article)
            card = cls._card_block(post)
            if card:
                blocks.append(card)

        quoted = cls._quoted_block(post)
        if quoted:
            blocks += ["", quoted]

        return _collapse_blank_lines("\n".join(blocks)), images

    @classmethod
    def _author_block(cls, user: User | None, core_user: dict[str, Any] | None) -> str:
        if user is None:
            return ""
        lines: list[str] = []
        name = _sanitize(user.name or "")
        handle = (user.handle or "").strip()
        norm = handle if handle.startswith("@") else f"@{handle}"
        if name and handle:
            lines.append(f"Author: {name} ({norm})")
        elif handle:
            lines.append(f"Author: {norm}")
        elif name:
            lines.append(f"Author: {name}")

        verified = cls._verified_label(core_user)
        if verified:
            lines.append(f"Verified: {verified}")
        label = cls._account_label(core_user)
        if label:
            lines.append(f"Account label: {label}")
        if user.is_protected:
            lines.append("Protected account")
        safety = cls._safety_label(core_user, user)
        if safety:
            lines.append(f"Safety: {safety}")

        bio = _truncate(user.bio or "", MAX_BIO)
        if bio:
            lines.append(f"Bio: {bio}")

        seg: list[str] = []
        f = _count_with_tier(user.follower_count)
        if f:
            seg.append(f"Followers: {f}")
        g = _count_with_tier(user.following_count)
        if g:
            seg.append(f"Following: {g}")
        sub = (user.subscription_level or "").strip()
        if sub:
            seg.append(sub)
        joined = _format_join_date(
            int(user.created_at.timestamp() * 1000) if user.created_at else None
        )
        if joined:
            seg.append(f"Joined {joined}")
        country = (user.signup_country_code or user.tfe_top_country or "").strip()
        if country:
            seg.append(country)
        lang = (user.account_lang or "").strip()
        if lang:
            seg.append(lang)
        if seg:
            lines.append(" \u00b7 ".join(seg))

        business = (
            _sanitize(user.affiliated_business.name or "")
            if user.affiliated_business
            else ""
        )
        if business:
            lines.append(f"Affiliated with: {business}")
        return "\n".join(lines)

    @staticmethod
    def _verified_label(core_user: dict[str, Any] | None) -> str:
        if not core_user:
            return ""
        safety = core_user.get("safety") or {}
        vt = safety.get("verifiedType")
        sub = (
            (core_user.get("account") or {}).get("subscriptionTier")
            or (core_user.get("profile") or {}).get("subscriptionLevel")
            or ""
        )
        color = None
        if vt == "Government":
            color = "Grey"
        elif vt == "Business":
            color = "Gold"
        elif core_user.get("isBlueVerified") or safety.get("isBlueVerified"):
            color = "Blue"
        if not color:
            return ""
        return f"{color} ({sub})" if sub else color

    @staticmethod
    def _account_label(core_user: dict[str, Any] | None) -> str:
        if not core_user:
            return ""
        lab = (
            (core_user.get("account") or {}).get("parodyCommentaryFanLabel") or ""
        ).strip()
        return lab if lab and lab.lower() != "none" else ""

    @staticmethod
    def _safety_label(core_user: dict[str, Any] | None, user: User | None) -> str:
        flags: list[str] = []
        if core_user:
            safety = core_user.get("safety") or {}
            if safety.get("nsfwUser") or safety.get("nsfwAdmin"):
                flags.append("NSFW")
        return ", ".join(flags)

    @classmethod
    def _collect_media_records(
        cls, post: Post, max_frames: int, is_own: bool
    ) -> list[RenderedMedium]:
        records: list[RenderedMedium] = []
        if not post.media:
            return records
        for medium in post.media:
            if (
                isinstance(medium, Image)
                and medium.convo_image
                and medium.convo_image.content
            ):
                records.append(
                    RenderedMedium(
                        label="Post image:",
                        quoted_label="Quoted post image:",
                        frames=[medium.convo_image.content],
                        transcript=None,
                        is_video=False,
                        is_own=is_own,
                    )
                )
            elif (
                isinstance(medium, Video)
                and medium.convo_video
                and medium.convo_video.frames
            ):
                frames = [f for f in medium.convo_video.frames if f][:max_frames]
                if not frames:
                    continue
                vi = medium.videoInfo or medium.animatedGifInfo
                dur = _humanize_duration(vi.durationMillis if vi else None)
                qual = _quality_bucket(vi.get_highest_bitrate() if vi else None)
                if dur and qual:
                    suffix = f"(duration {dur}, quality {qual}, {len(frames)} uniformly sampled frames):"
                else:
                    suffix = f"({len(frames)} uniformly sampled frames):"
                records.append(
                    RenderedMedium(
                        label=f"Post video {suffix}",
                        quoted_label=f"Quoted post video {suffix}",
                        frames=frames,
                        transcript=(medium.convo_video.asr_transcript or None),
                        is_video=True,
                        is_own=is_own,
                    )
                )
        return records

    @staticmethod
    def _apply_frame_budget(
        records: list[RenderedMedium], max_media: int
    ) -> list[RenderedMedium]:
        out: list[RenderedMedium] = []
        used = 0
        for rec in records:
            if used >= max_media:
                break
            take = min(len(rec.frames), max_media - used)
            if take <= 0:
                continue
            if take < len(rec.frames):
                old_count = f"{len(rec.frames)} uniformly sampled frames"
                new_count = f"{take} uniformly sampled frames"
                rec = replace(
                    rec,
                    frames=rec.frames[:take],
                    label=rec.label.replace(old_count, new_count),
                    quoted_label=rec.quoted_label.replace(old_count, new_count),
                )
            out.append(rec)
            used += take
        return out

    @staticmethod
    def _article_block(post: Post) -> str:
        am = getattr(post, "article_metadata", None)
        title = (getattr(am, "title", None) or "").strip() if am else ""
        return f"Article: {_truncate(title, MAX_ARTICLE_TITLE)}" if title else ""

    @staticmethod
    def _card_block(post: Post) -> str:
        if not post.cardsV2:
            return ""
        for c in post.cardsV2:
            lc = getattr(c, "legacy_card", None)
            title = (getattr(lc, "title", None) or "").strip() if lc else ""
            domain = (lc.domain or "").strip() if lc else ""
            if title:
                dom = (
                    re.sub(r"^https?://(www\.)?", "", domain).split("/")[0]
                    if domain
                    else ""
                )
                t = _truncate(title, MAX_CARD_TITLE)
                return f"Link card: {t} ({dom})" if dom else f"Link card: {t}"
        return ""

    @classmethod
    def _quoted_block(cls, post: Post) -> str:
        qp = getattr(post, "quoted_post", None)
        if qp is None:
            return ""
        qtext = _truncate(qp.full_text or "", MAX_QUOTED_TEXT)
        qu = qp.user
        attribution = ""
        if qu:
            name = _sanitize(qu.name or "")
            handle = (qu.handle or "").strip()
            norm = handle if handle.startswith("@") else f"@{handle}"
            stats = []
            f = _count_with_tier(qu.follower_count)
            if f:
                stats.append(
                    f"{f.split(' ')[0]} followers ({_tier_label(qu.follower_count)})"
                    if _tier_label(qu.follower_count)
                    else f"{f} followers"
                )
            g = _count_with_tier(qu.following_count)
            if g:
                stats.append(f"Following: {g}")
            inner = norm + (", " + " \u00b7 ".join(stats) if stats else "")
            attribution = f"{name} ({inner})" if name else f"({inner})"
        head = (
            f"This post quotes a post by {attribution}"
            if attribution
            else "This post quotes a post"
        )
        if qtext:
            return f"{head}:\nQuoted post text: {qtext}"
        return f"{head}."
