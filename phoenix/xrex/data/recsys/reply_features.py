# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from __future__ import annotations

import enum

import numpy as np
import numpy.typing as npt
import pyarrow as pa

from xrex.data.recsys.feature_config import CategoricalFeature, Int64Feature

TWITTER_EPOCH_MS = 1288834974657

REPLY_INJECTED_NEGATIVE_INT64_INDEX = 2
REPLY_HAS_MEDIA_INT64_INDEX = 4
REPLY_FAV_COUNT_INT64_INDEX = Int64Feature.favCountSeq.value
REPLY_REPLY_COUNT_INT64_INDEX = Int64Feature.replyCountSeq.value
REPLY_REPOST_COUNT_INT64_INDEX = Int64Feature.repostCountSeq.value
REPLY_QUOTE_COUNT_INT64_INDEX = Int64Feature.quoteCountSeq.value
REPLY_VIEW_COUNT_INT64_INDEX = Int64Feature.viewCountSeq.value
REPLY_VIDEO_DURATION_MS_INT64_INDEX = 13
REPLY_AUTHOR_FOLLOWER_COUNT_INT64_INDEX = 14
REPLY_NEGFB_TOTAL_COUNT_INT64_INDEX = 15
REPLY_FOCAL_AUTHOR_FOLLOWS_INT64_INDEX = 16
REPLY_SECTION_INT64_INDEX = 17
REPLY_GROX_SCORE_INT64_INDEX = 18
REPLY_AUTHOR_VERIFIED_INT64_INDEX = 19
REPLY_FAV_BY_ROOT_AUTHOR_INT64_INDEX = 20
REPLY_FOCAL_AUTHOR_REPLY_INT64_INDEX = 21
REPLY_SELF_REPLY_INT64_INDEX = 22
REPLY_POST_INT64_FEATURE_SIZE = REPLY_SELF_REPLY_INT64_INDEX + 1

REPLY_INT64_COLUMNS: dict[str, int] = {
    "isInjectedNegativeSeq": REPLY_INJECTED_NEGATIVE_INT64_INDEX,
    "hasMediaSeq": REPLY_HAS_MEDIA_INT64_INDEX,
    "firstVideoDurationMsSeq": REPLY_VIDEO_DURATION_MS_INT64_INDEX,
    "authorFollowerCountSeq": REPLY_AUTHOR_FOLLOWER_COUNT_INT64_INDEX,
    "negfbTotalCountSeq": REPLY_NEGFB_TOTAL_COUNT_INT64_INDEX,
    "focalAuthorFollowsSeq": REPLY_FOCAL_AUTHOR_FOLLOWS_INT64_INDEX,
    "sectionSeq": REPLY_SECTION_INT64_INDEX,
    "groxScoreSeq": REPLY_GROX_SCORE_INT64_INDEX,
    "authorVerifiedSeq": REPLY_AUTHOR_VERIFIED_INT64_INDEX,
    "favByRootAuthorSeq": REPLY_FAV_BY_ROOT_AUTHOR_INT64_INDEX,
    "focalAuthorReplySeq": REPLY_FOCAL_AUTHOR_REPLY_INT64_INDEX,
    "selfReplySeq": REPLY_SELF_REPLY_INT64_INDEX,
}
REPLY_SATURATE_INT64_INDICES = (
    REPLY_AUTHOR_FOLLOWER_COUNT_INT64_INDEX,
    REPLY_NEGFB_TOTAL_COUNT_INT64_INDEX,
    REPLY_GROX_SCORE_INT64_INDEX,
)

REPLY_POST_LANGUAGE_CATEGORICAL_INDEX = 4
REPLY_CATEGORICAL_COLUMNS: dict[str, int] = {
    "postLanguageCodeSeq": REPLY_POST_LANGUAGE_CATEGORICAL_INDEX
}
assert REPLY_POST_LANGUAGE_CATEGORICAL_INDEX not in {f.value for f in CategoricalFeature}


class ReplyUserInt64Feature(enum.IntEnum):
    focalPostCreationTsSec = 0
    focalFavCount = 1
    focalReplyCount = 2
    focalViewCount = 3
    focalAuthorFollowerCount = 4
    focalPostLanguageCode = 5
    viewerIsFocalAuthor = 6
    focalInViewerHistory = 7


REPLY_USER_INT64_FEATURE_SIZE = max(ReplyUserInt64Feature) + 1

REPLY_USER_INT64_COLUMNS = (
    ReplyUserInt64Feature.focalFavCount,
    ReplyUserInt64Feature.focalReplyCount,
    ReplyUserInt64Feature.focalViewCount,
    ReplyUserInt64Feature.focalAuthorFollowerCount,
    ReplyUserInt64Feature.focalPostLanguageCode,
)

REPLY_OPTIONAL_COLUMNS: list[str] = [
    "inReplyToPostIdSeq",
    "inReplyToAuthorIdSeq",
    "inReplyToSemanticIdSeq",
    "tweetPositionSeq",
    *REPLY_INT64_COLUMNS,
    *REPLY_CATEGORICAL_COLUMNS,
    *(f.name for f in REPLY_USER_INT64_COLUMNS),
]


def _seq_col(rb: pa.RecordBatch, name: str, batch_size: int, dtype: type) -> npt.NDArray:
    values = rb.column(name).values
    if values.null_count:
        values = values.fill_null(False if pa.types.is_boolean(values.type) else 0)
    return values.to_numpy(zero_copy_only=False).reshape(batch_size, -1).astype(dtype)


def read_candidate_columns(
    rb: pa.RecordBatch, batch_size: int
) -> tuple[dict[int, npt.NDArray[np.int64]], dict[int, npt.NDArray[np.int16]]]:
    names = set(rb.schema.names)
    int64 = {
        slot: _seq_col(rb, col, batch_size, np.int64)
        for col, slot in REPLY_INT64_COLUMNS.items()
        if col in names
    }
    categorical = {
        slot: _seq_col(rb, col, batch_size, np.int16)
        for col, slot in REPLY_CATEGORICAL_COLUMNS.items()
        if col in names
    }
    return int64, categorical


def read_user_int64(rb: pa.RecordBatch, batch_size: int) -> npt.NDArray[np.int64]:
    out = np.zeros((batch_size, REPLY_USER_INT64_FEATURE_SIZE), dtype=np.int64)
    for feat in REPLY_USER_INT64_COLUMNS:
        if feat.name in rb.schema.names:
            arr = rb.column(feat.name)
            if arr.null_count:
                arr = arr.fill_null(0)
            out[:, feat.value] = np.clip(
                arr.to_numpy(zero_copy_only=False).astype(np.int64), 0, 2**31 - 1
            )
    return out


def read_parent_sids(rb: pa.RecordBatch, batch_size: int) -> npt.NDArray[np.int32] | None:
    if "inReplyToSemanticIdSeq" not in rb.schema.names:
        return None
    col = rb.column("inReplyToSemanticIdSeq")
    levels = col.type.value_type.list_size
    return (
        col.values.flatten()
        .to_numpy(zero_copy_only=False)
        .reshape(batch_size, col.type.list_size, levels)
        .astype(np.int32)
    )


def place_focal_post(
    *,
    rb: pa.RecordBatch,
    batch_size: int,
    new_event_mask: npt.NDArray[np.bool_],
    padding_mask: npt.NDArray[np.bool_],
    user_ids: npt.NDArray,
    history_post_ids: npt.NDArray,
    history_auth_ids: npt.NDArray,
    history_post_sids: npt.NDArray[np.uint16] | None,
    user_int64: npt.NDArray[np.int64],
) -> None:
    names = set(rb.schema.names)
    if "inReplyToPostIdSeq" in names:
        parent_ids = _seq_col(rb, "inReplyToPostIdSeq", batch_size, np.int64)
        parent_authors = (
            _seq_col(rb, "inReplyToAuthorIdSeq", batch_size, np.int64)
            if "inReplyToAuthorIdSeq" in names
            else None
        )
        for i in range(batch_size):
            cand = np.where(new_event_mask[i] & padding_mask[i])[0]
            if len(cand) == 0:
                continue
            first = int(cand[0])
            focal_post_id = int(parent_ids[i, first])
            focal_author_id = int(parent_authors[i, first]) if parent_authors is not None else 0
            if focal_post_id > 0:
                user_int64[i, ReplyUserInt64Feature.focalPostCreationTsSec] = (
                    (focal_post_id >> 22) + TWITTER_EPOCH_MS
                ) // 1000
                user_int64[i, ReplyUserInt64Feature.focalInViewerHistory] = (
                    2 if np.any(history_post_ids[i] == focal_post_id) else 1
                )
            history_post_ids[i, 0] = focal_post_id
            history_auth_ids[i, 0] = focal_author_id
            if focal_author_id > 0:
                user_int64[i, ReplyUserInt64Feature.viewerIsFocalAuthor] = (
                    2 if int(user_ids[i]) == focal_author_id else 1
                )
    if history_post_sids is not None:
        history_post_sids[:, 0, :] = 0
        parent_sids = read_parent_sids(rb, batch_size)
        if parent_sids is not None and parent_sids.shape[-1] == history_post_sids.shape[-1]:
            history_post_sids[:, 0, :] = (parent_sids[:, 0, :] + 1).astype(np.uint16)
