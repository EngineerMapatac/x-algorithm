#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.

from __future__ import annotations

import base64
import sys
from pathlib import Path

_REF = Path(__file__).resolve().parent.parent
if str(_REF) not in sys.path:
    sys.path.insert(0, str(_REF))

from mm_encoder import (
    Author,
    CardV2,
    Image,
    LegacyCard,
    Post,
    Video,
)

TINY_PNG = base64.b64decode(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAC0lEQVR4nGNgYGAAAAAEAAH2FzhVAAAAAElFTkSuQmCC"
)


def _png(tag: bytes) -> bytes:
    return TINY_PNG + tag


def example_posts() -> list[Post]:
    return [
        Post(
            id=10_001,
            text="just shipped the open-source recsys reference https://t.co/abc123",
            author=Author(name="Open Source", handle="oss"),
        ),
        Post(
            id=10_002,
            text="a chart of the loss curve",
            author=Author(name="ML Eng", handle="mleng"),
            media=[Image(content=_png(b"a"))],
        ),
        Post(
            id=10_003,
            text="before and after",
            author=Author(name="Designer", handle="dsgn"),
            media=[Image(content=_png(b"before")), Image(content=_png(b"after"))],
        ),
        Post(
            id=10_004,
            text="great thread, adding my take",
            author=Author(name="Commenter", handle="cmt"),
            media=[Image(content=_png(b"main"))],
            quoted_post=Post(
                id=10_005,
                text="original insight here",
                author=Author(name="Author", handle="orig"),
                media=[Image(content=_png(b"quoted"))],
            ),
        ),
        Post(
            id=10_006,
            text="clip from the talk",
            author=Author(name="Speaker", handle="spkr"),
            media=[Video(frames=[_png(b"f0"), _png(b"f1"), _png(b"f2"), _png(b"f3")])],
        ),
        Post(
            id=10_007,
            text="good read",
            author=Author(name="Curator", handle="cur"),
            cards=[
                CardV2(
                    legacy_card=LegacyCard(
                        title="Scaling Retrieval",
                        description="how we built it",
                        thumbnail_image=Image(content=_png(b"card")),
                    )
                )
            ],
        ),
    ]


def _self_check() -> None:
    from mm_encoder import IMAGE_PAD, format_chatml, render_post

    posts = example_posts()
    assert len(posts) == 6, len(posts)
    for p in posts:
        text, images = render_post(p)
        assert text.count(IMAGE_PAD) == len(images), (
            f"post {p.id}: {text.count(IMAGE_PAD)} pads != {len(images)} images"
        )
        for b in images:
            assert b.startswith(TINY_PNG[:8]), f"post {p.id}: image not a PNG"
        cm = format_chatml(text)
        assert cm.endswith("assistant\n") and "<|im_start|>system" in cm

    quoted = posts[3]
    t, imgs = render_post(quoted, max_images=2)
    assert "\n\nQuoted post:" in t, t
    assert len(imgs) == 2 and t.count(IMAGE_PAD) == 2, (len(imgs), t.count(IMAGE_PAD))
    assert imgs[0] == _png(b"main") and imgs[1] == _png(b"quoted"), "quoted order wrong"

    counts = {p.id: render_post(p)[0].count(IMAGE_PAD) for p in posts}
    print(
        "example_posts self-check PASS  "
        f"posts={len(posts)} pads_per_post={counts}\n"
        "  (⚠ synthetic example data — NOT real X posts)"
    )


if __name__ == "__main__":
    _self_check()
