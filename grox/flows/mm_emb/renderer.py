import re

import numpy as np

from grox.core.data_loaders.data_types import CardV2, Image, Post, Video

IMAGE_PAD = "<|vision_start|><|image_pad|><|vision_end|>"


def remove_tco_links(text: str) -> str:
    return re.sub(r"https?://t\.co/\S+", "", text).strip()


class V5EmbedPostRenderer:
    @classmethod
    def render_for_embedding(
        cls,
        post: Post,
        max_images: int | None = None,
        include_quoted: bool = True,
    ) -> tuple[str, list[bytes]]:
        num_posts = 1
        if include_quoted and post.quoted_post:
            num_posts = 2

        per_post_budget = None
        if max_images is not None:
            per_post_budget = max_images // num_posts

        text_parts: list[str] = []
        images: list[bytes] = []

        cls._render_single_post(post, text_parts, images, per_post_budget)

        if include_quoted and post.quoted_post:
            text_parts.append("\n\nQuoted post:")
            quoted_text_parts: list[str] = []
            quoted_images: list[bytes] = []
            cls._render_single_post(
                post.quoted_post, quoted_text_parts, quoted_images, per_post_budget
            )
            if quoted_text_parts:
                text_parts.extend(quoted_text_parts)
            images.extend(quoted_images)

        return "\n".join(filter(None, text_parts)), images

    @classmethod
    def _render_single_post(
        cls,
        post: Post,
        text_parts: list[str],
        images: list[bytes],
        max_images: int | None,
    ) -> None:
        if post.user:
            user_parts: list[str] = []
            if post.user.name:
                user_parts.append(post.user.name)
            if post.user.handle:
                user_parts.append(f"@{post.user.handle}")
            if user_parts:
                text_parts.append(" ".join(user_parts))

        if post.article_metadata:
            if post.article_metadata.title:
                text_parts.append(f"Article: {post.article_metadata.title}")
            if post.article_metadata.cover_media:
                cls._process_image(
                    post.article_metadata.cover_media, text_parts, images, max_images
                )

        if post.full_text:
            text_parts.append(remove_tco_links(post.full_text))

        if post.media:
            for medium in post.media:
                if max_images is not None and len(images) >= max_images:
                    break
                cls._process_medium(medium, text_parts, images, max_images)

        if post.cardsV2:
            for cardV2 in post.cardsV2:
                if max_images is not None and len(images) >= max_images:
                    break
                cls._process_cardv2(cardV2, text_parts, images, max_images)

        if post.article_metadata and post.article_metadata.media:
            for medium in post.article_metadata.media:
                if max_images is not None and len(images) >= max_images:
                    break
                cls._process_medium(medium, text_parts, images, max_images)

    @classmethod
    def _process_medium(
        cls,
        medium: Image | Video,
        text_parts: list[str],
        images: list[bytes],
        max_images: int | None,
    ) -> None:
        if isinstance(medium, Image):
            cls._process_image(medium, text_parts, images, max_images)
        elif isinstance(medium, Video):
            cls._process_video(medium, text_parts, images, max_images)

    @classmethod
    def _process_image(
        cls,
        image: Image,
        text_parts: list[str],
        images: list[bytes],
        max_images: int | None,
    ) -> None:
        if max_images is not None and len(images) >= max_images:
            return
        if image.convo_image and image.convo_image.content:
            text_parts.append(IMAGE_PAD)
            images.append(image.convo_image.content)

    @classmethod
    def _process_video(
        cls,
        video: Video,
        text_parts: list[str],
        images: list[bytes],
        max_images: int | None,
    ) -> None:
        if not video.convo_video or not video.convo_video.frames:
            return

        frames = [f for f in video.convo_video.frames if f]
        if not frames:
            return

        remaining_budget = None
        if max_images is not None:
            remaining_budget = max_images - len(images)
            if remaining_budget <= 0:
                return

        if remaining_budget is not None and len(frames) > remaining_budget:
            indices = np.linspace(
                0, len(frames) - 1, remaining_budget, dtype=int
            ).tolist()
            frames = [frames[i] for i in indices]

        for frame in frames:
            if max_images is not None and len(images) >= max_images:
                break
            text_parts.append(IMAGE_PAD)
            images.append(frame)

    @classmethod
    def _process_cardv2(
        cls,
        cardV2: CardV2,
        text_parts: list[str],
        images: list[bytes],
        max_images: int | None,
    ) -> None:
        if cardV2.legacy_card:
            lc = cardV2.legacy_card
            card_text = []
            if lc.title:
                card_text.append(f"Card: {lc.title}")
            if lc.description:
                card_text.append(lc.description)
            if card_text:
                text_parts.append("\n" + "\n".join(card_text))

            if lc.thumbnail_image:
                cls._process_image(lc.thumbnail_image, text_parts, images, max_images)

            if lc.poll_cards:
                for poll_card in lc.poll_cards:
                    if poll_card.choice_label:
                        text_parts.append(f"Poll choice: {poll_card.choice_label}")
                    if poll_card.choice_image:
                        cls._process_image(
                            poll_card.choice_image, text_parts, images, max_images
                        )

        if cardV2.unified_cards:
            for uc in cardV2.unified_cards:
                card_text = []
                if uc.title:
                    card_text.append(f"Card: {uc.title}")
                if uc.description:
                    card_text.append(uc.description)
                if card_text:
                    text_parts.append("\n" + "\n".join(card_text))

                if uc.media:
                    cls._process_medium(uc.media, text_parts, images, max_images)
