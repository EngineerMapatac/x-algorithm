# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from __future__ import annotations

import os
import re

import numpy as np
import pyarrow as pa
import pyarrow.parquet as pq

DELAY_COLUMN = "conversionDelayMsSeq"

ACTION_DELAY_PREFIX = "conversionActionDelayMsSeq_"
ACTION_MULTIHOT_COLUMN = "actionNameMultiHotSeqSeq"

DEFAULT_WINDOW_MS = 7 * 24 * 60 * 60 * 1000

_PARTITION_SEG_RE = re.compile(r"(partition=\d+/)")


def type_delay_column(conversion_type: str) -> str:
    sanitized = "".join(c if c.isalnum() else "_" for c in conversion_type)
    return f"{DELAY_COLUMN}_{sanitized}"


def action_delay_columns(sidecar_path: str) -> list[str]:
    names = pq.ParquetFile(sidecar_path).schema_arrow.names
    return sorted(
        (n for n in names if n.startswith(ACTION_DELAY_PREFIX)),
        key=lambda n: int(n[len(ACTION_DELAY_PREFIX) :]),
    )


def action_index_of(column: str) -> int:
    if not column.startswith(ACTION_DELAY_PREFIX):
        raise ValueError(f"not an action delay column: {column}")
    return int(column[len(ACTION_DELAY_PREFIX) :])


def sidecar_path_for(batch_file_path: str) -> str:
    m = _PARTITION_SEG_RE.search(batch_file_path)
    if m is None:
        raise ValueError(f"not a partition batch file path: {batch_file_path}")
    root = batch_file_path[: m.start()]
    rel = batch_file_path[m.start() :]
    if not rel.endswith(".parquet"):
        raise ValueError(f"not a parquet path: {batch_file_path}")
    return os.path.join(root, "labels", rel[: -len(".parquet")] + ".labels.parquet")


def load_sidecar_delays(
    sidecar_path: str, columns: list[str] | None = None
) -> dict[str, np.ndarray]:
    columns = columns or [DELAY_COLUMN]
    schema_names = pq.ParquetFile(sidecar_path).schema_arrow.names
    missing = [c for c in columns if c not in schema_names]
    if missing:
        raise ValueError(
            f"sidecar {sidecar_path} missing column(s) {missing}; available: {schema_names}"
        )
    t = pq.read_table(sidecar_path, columns=columns)
    out: dict[str, np.ndarray] = {}
    for name in columns:
        col = t.column(name).combine_chunks()
        if isinstance(col, pa.ChunkedArray):
            col = col.chunk(0)
        seq_len = col.type.list_size
        flat = col.flatten().to_numpy(zero_copy_only=False).astype(np.int64)
        out[name] = flat.reshape(len(col), seq_len)
    return out


def attach_delays(batch: pa.RecordBatch, delays: dict[str, np.ndarray]) -> pa.RecordBatch:
    for name, mat in delays.items():
        if mat.shape[0] != batch.num_rows:
            raise ValueError(f"{name}: delay rows {mat.shape[0]} != batch rows {batch.num_rows}")
        arr = pa.FixedSizeListArray.from_arrays(
            pa.array(mat.reshape(-1), type=pa.int64()), mat.shape[1]
        )
        batch = batch.append_column(name, arr)
    return batch


def fold_action_delays_into_multihot(batch: pa.RecordBatch, window_ms: int) -> pa.RecordBatch:
    action_cols = [n for n in batch.schema.names if n.startswith(ACTION_DELAY_PREFIX)]
    if not action_cols:
        return batch
    col_idx = batch.schema.get_field_index(ACTION_MULTIHOT_COLUMN)
    if col_idx < 0:
        raise ValueError(f"batch missing {ACTION_MULTIHOT_COLUMN}")
    mh = batch.column(col_idx)
    seq_len = mh.type.list_size
    vocab = mh.type.value_type.list_size
    bits = (
        mh.flatten()
        .flatten()
        .to_numpy(zero_copy_only=False)
        .astype(np.bool_)
        .reshape(batch.num_rows, seq_len, vocab)
        .copy()
    )
    mask_name = next(
        (n for n in ("newEventMask", "newEventMaskSeq") if n in batch.schema.names), None
    )
    if mask_name is None:
        raise ValueError("batch missing newEventMask; conversion-label folding is candidate-only")
    mask_col = batch.column(mask_name)
    is_candidate = (
        mask_col.flatten()
        .to_numpy(zero_copy_only=False)
        .astype(np.bool_)
        .reshape(batch.num_rows, mask_col.type.list_size)
    )
    if is_candidate.shape[1] != seq_len:
        raise ValueError(f"{mask_name}: seq len {is_candidate.shape[1]} != multi-hot {seq_len}")
    for name in action_cols:
        idx = action_index_of(name)
        if idx >= vocab:
            raise ValueError(f"{name}: bit {idx} outside vocab {vocab}")
        delays_col = batch.column(name)
        delays = (
            delays_col.flatten()
            .to_numpy(zero_copy_only=False)
            .astype(np.int64)
            .reshape(batch.num_rows, delays_col.type.list_size)
        )
        if delays.shape[1] != seq_len:
            raise ValueError(f"{name}: seq len {delays.shape[1]} != multi-hot {seq_len}")
        bits[:, :, idx] = np.where(
            is_candidate, delays_to_labels(delays, window_ms), bits[:, :, idx]
        )
    inner = pa.FixedSizeListArray.from_arrays(pa.array(bits.reshape(-1)), vocab)
    outer = pa.FixedSizeListArray.from_arrays(inner, seq_len)
    return batch.set_column(col_idx, batch.schema.field(col_idx), outer)


def delays_to_labels(delays: np.ndarray, window_ms: int) -> np.ndarray:
    if window_ms < 0:
        raise ValueError(f"window_ms must be >= 0, got {window_ms}")
    return (delays >= 0) & (delays <= window_ms)
