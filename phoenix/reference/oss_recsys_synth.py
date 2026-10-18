#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.

from __future__ import annotations

import numpy as np

_MISSING_CODE = 0


def synth_mm_embeddings(
    ids: np.ndarray,
    dim: int,
    *,
    seed: int = 0,
    normalize: bool = True,
) -> np.ndarray:
    rng = np.random.default_rng(seed)
    a = rng.standard_normal(dim).astype(np.float64) * 0.02
    b = rng.uniform(0.0, 2.0 * np.pi, size=dim).astype(np.float64)
    x = ids.astype(np.float64)[..., None] * a + b
    emb = np.sin(x).astype(np.float32)
    if normalize:
        norm = np.linalg.norm(emb, axis=-1, keepdims=True)
        emb = emb / np.maximum(norm, 1e-6)
    return emb


def synth_sid_codebooks(
    num_levels: int,
    codebook_size: int,
    dim: int,
    *,
    seed: int = 1,
) -> np.ndarray:
    rng = np.random.default_rng(seed)
    book = rng.standard_normal((num_levels, codebook_size, dim)).astype(np.float32)
    book /= np.maximum(np.linalg.norm(book, axis=-1, keepdims=True), 1e-6)
    return book


def mm_to_sid(mm_emb: np.ndarray, codebooks: np.ndarray) -> np.ndarray:
    num_levels, codebook_size, dim = codebooks.shape
    assert mm_emb.shape[-1] == dim, (mm_emb.shape, dim)
    lead = mm_emb.shape[:-1]
    residual = mm_emb.reshape(-1, dim).astype(np.float32)
    codes = np.empty((residual.shape[0], num_levels), dtype=np.int32)
    for level in range(num_levels):
        book = codebooks[level]
        d = (
            (residual * residual).sum(-1, keepdims=True)
            - 2.0 * residual @ book.T
            + (book * book).sum(-1)[None, :]
        )
        idx = np.argmin(d, axis=1)
        codes[:, level] = idx.astype(np.int32)
        residual = residual - book[idx]
    return codes.reshape(*lead, num_levels)


def sid_codes_for_posts(
    ids: np.ndarray,
    num_levels: int,
    codebook_size: int,
    *,
    mm_dim: int = 1024,
    seed: int = 0,
) -> np.ndarray:
    mm = synth_mm_embeddings(ids, mm_dim, seed=seed)
    codebooks = synth_sid_codebooks(num_levels, codebook_size, mm_dim, seed=seed + 1)
    return mm_to_sid(mm, codebooks)


def synth_post_sids(
    ids: np.ndarray,
    num_levels: int,
    codebook_size: int,
    *,
    mm_dim: int = 1024,
    seed: int = 0,
) -> np.ndarray:
    wire = sid_codes_for_posts(ids, num_levels, codebook_size, mm_dim=mm_dim, seed=seed)
    codes = (wire + 1).astype(np.uint16)
    codes = np.where((ids == 0)[..., None], np.uint16(_MISSING_CODE), codes)
    return codes.astype(np.uint16)


if __name__ == "__main__":
    ids = np.arange(0, 10, dtype=np.int64)
    mm = synth_mm_embeddings(ids, 1024)
    wire = sid_codes_for_posts(ids, num_levels=6, codebook_size=256)
    buf = synth_post_sids(ids, num_levels=6, codebook_size=256)
    assert mm.shape == (10, 1024) and mm.dtype == np.float32
    assert wire.shape == (10, 6) and wire.dtype == np.int32
    assert (wire >= 0).all() and (wire < 256).all(), "wire codes out of [0,256)"
    assert (sid_codes_for_posts(ids, 6, 256) == wire).all(), "wire not deterministic"
    assert buf.shape == (10, 6) and buf.dtype == np.uint16
    assert (synth_post_sids(ids, 6, 256) == buf).all(), "buffer not deterministic"
    assert (buf[1:] >= 1).all() and (buf[1:] <= 256).all(), "buffer codes out of [1,256]"
    assert (buf[0] == 0).all(), "id==0 must be all-missing"
    assert (buf[1:] == wire[1:] + 1).all(), "buffer must be wire+1"
    assert len({tuple(r) for r in wire[1:]}) > 1, "SIDs collapsed"
    print("oss_recsys_synth self-check PASS  wire:", wire[1].tolist(), " buffer:", buf[1].tolist())
