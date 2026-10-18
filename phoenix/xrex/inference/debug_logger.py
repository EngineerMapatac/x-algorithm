# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import pyarrow as pa

USER_ACTIONS_FIELDS_SINGLE = ["user_ids", "post_lengths"]
USER_ACTIONS_FIELDS_DOUBLE = ["author_ids", "tweet_ids", "impressed_time_ms"]
USER_ACTIONS_FIELDS_BOOL_TRIPLE = ["actions"]
CANDIDATE_SET_FIELDS_SINGLE = ["can_lengths"]
CANDIDATE_SET_FIELDS_DOUBLE = ["author_ids", "tweet_ids"]
BATCH_FIELDS_DOUBLE = [
    "inputs",
    "impression_timestamps",
    "creation_timestamps",
    "candidate_tokens",
    "token_counts",
]
BATCH_FIELDS_BOOL_DOUBLE = [
    "padding_mask",
]


def build_debug_schema() -> pa.Schema:
    schema = []
    for field in USER_ACTIONS_FIELDS_SINGLE:
        schema.append(pa.field(f"input_{field}", pa.int64()))
    for field in USER_ACTIONS_FIELDS_DOUBLE:
        schema.append(pa.field(f"input_{field}", pa.list_(pa.int64())))
    for field in USER_ACTIONS_FIELDS_BOOL_TRIPLE:
        schema.append(pa.field(f"input_{field}", pa.list_(pa.list_(pa.bool_()))))
    for field in CANDIDATE_SET_FIELDS_SINGLE:
        schema.append(pa.field(f"candidate_{field}", pa.int64()))
    for field in CANDIDATE_SET_FIELDS_DOUBLE:
        schema.append(pa.field(f"candidate_{field}", pa.list_(pa.int64())))
    for field in BATCH_FIELDS_DOUBLE:
        schema.append(pa.field(f"batch_{field}", pa.list_(pa.int64())))
    for field in BATCH_FIELDS_BOOL_DOUBLE:
        schema.append(pa.field(f"batch_{field}", pa.list_(pa.bool_())))
    return pa.schema(schema)
