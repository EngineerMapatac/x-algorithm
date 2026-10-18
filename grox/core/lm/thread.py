import logging

from grox.core.data_loaders.data_types import Post
from grox.core.lm.convo import SEPARATOR, Content, Message, Role
from grox.core.lm.post import PostRenderer
from grox.core.lm.user import UserRenderer

logger = logging.getLogger(__name__)
MAX_THREAD_SIZE = 10
MAX_MEDIA_PER_POST = 2


class ThreadRenderer:
    @classmethod
    def render(
        cls,
        post: Post,
        role: Role = Role.USER,
        separator=SEPARATOR,
        include_signals: bool = False,
        include_follower_count: bool = False,
    ) -> Message:
        message = Message(role=role, content=[], separator=separator)
        message.content.append(
            "\n# Reply Author Info\n\nThe user profile and any signals below describe the author of the reply being evaluated (the final post of the thread).\n"
        )
        message.content.extend(UserRenderer.render(post.user))
        if include_signals:
            message.content.extend(cls._render_signals(post))
        message.content.extend(
            cls._render_thread(post, include_follower_count=include_follower_count)
        )
        return message

    @classmethod
    def _render_signals(cls, post: Post) -> list[Content]:
        lines: list[str] = []
        user = post.user
        if user:
            if user.has_missing_client_events is not None:
                lines.append(
                    f"  - Has Missing Client Events: {user.has_missing_client_events}"
                )
            if user.tfe_top_country:
                lines.append(f"  - TFE Top Country: {user.tfe_top_country}")
            if user.account_lang:
                lines.append(f"  - Account Language: {user.account_lang}")
            if user.num_replies_last_24hrs is not None:
                lines.append(
                    f"  - Num Replies Last 24 Hours: {user.num_replies_last_24hrs:.0f}"
                )
            if user.follower_count is not None:
                lines.append(f"  - Follower Count: {user.follower_count}")
            if user.has_risky_user_safety_label is not None:
                lines.append(
                    f"  - Has Risky User Safety Label: {user.has_risky_user_safety_label}"
                )
            if user.num_legit_blocks_received_last_24hrs is not None:
                lines.append(
                    f"  - Num Legit Blocks Received Last 24 Hours: {user.num_legit_blocks_received_last_24hrs:.1f}"
                )
        if post.is_pasted is not None:
            lines.append(f"  - Reply Was Pasted: {post.is_pasted}")
        if not lines:
            return []
        return ["\n# Additional Signals\n" + "\n".join(lines) + "\n"]

    @classmethod
    def _render_thread(
        cls, post: Post, include_follower_count: bool = False
    ) -> list[Content]:
        res = []
        res.append(
            "\n\n# Thread \n\nListed below is the X post thread before the reply. Post 0 is the original post. \n\n"
        )
        ancestors = post.ancestors or []
        if len(ancestors) > MAX_THREAD_SIZE:
            ancestors = (
                ancestors[: MAX_THREAD_SIZE // 2] + ancestors[-(MAX_THREAD_SIZE // 2) :]
            )
        post_list = ancestors
        for post_idx, p in enumerate(post_list):
            res.append(f"\n\n#### Post {post_idx}\n\n")
            if p is None:
                res.append("\n<- A post has been deleted ->\n")
            else:
                res.extend(
                    PostRenderer.render(
                        p,
                        max_media=MAX_MEDIA_PER_POST,
                        include_follower_count=include_follower_count,
                        include_bio=include_follower_count and post_idx == 0,
                    )
                )
            res.append("\n\n------\n\n")

        res.append("\n\n# Reply \n\nBelow is the reply that you need to evaluate. \n\n")
        res.extend(
            PostRenderer.render(
                post,
                max_media=MAX_MEDIA_PER_POST,
                include_follower_count=include_follower_count,
            )
        )
        res.append("\n\n------\n\n")

        return res
