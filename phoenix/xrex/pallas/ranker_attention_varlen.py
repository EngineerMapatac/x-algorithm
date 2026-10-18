# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import dataclasses
import functools
from typing import Any

import jax
import jax.numpy as jnp
from jax import lax
from jax.ad_checkpoint import checkpoint_name
from jax.experimental import pallas as pl
from jax.experimental.pallas import triton as plgpu
from jax.experimental.pallas.triton import CompilerParams

DEFAULT_MASK_VALUE = -0.7 * float(jnp.finfo(jnp.dtype("float32")).max)


@dataclasses.dataclass(frozen=True)
class TuningConfig:
    block_q: int
    block_kv: int
    num_warps: int
    num_stages: int

    block_q_dq: int | None = None
    block_kv_dq: int | None = None
    block_q_dkv: int | None = None
    block_kv_dkv: int | None = None

    bwd_num_warps: int | None = None
    bwd_num_stages: int | None = None

    def __post_init__(self):
        backward_blocks = [self.block_q_dq, self.block_kv_dq, self.block_q_dkv, self.block_kv_dkv]
        block_is_set = [blk is not None for blk in backward_blocks]
        if any(block_is_set) and not all(block_is_set):
            raise ValueError(
                "Backward block sizes (block_q_dq, block_kv_dq, block_q_dkv, "
                "block_kv_dkv) must either all be specified or all be None."
            )


HOPPER_CONFIG = TuningConfig(
    block_q=128,
    block_kv=128,
    num_warps=8,
    num_stages=2,
    block_q_dq=128,
    block_kv_dq=16,
    block_q_dkv=16,
    block_kv_dkv=128,
    bwd_num_warps=8,
    bwd_num_stages=4,
)

BLACKWELL_CONFIG = TuningConfig(
    block_q=128,
    block_kv=64,
    num_warps=8,
    num_stages=2,
    block_q_dq=128,
    block_kv_dq=16,
    block_q_dkv=16,
    block_kv_dkv=64,
    bwd_num_warps=8,
    bwd_num_stages=4,
)


@functools.cache
def get_device_tuning_config() -> TuningConfig:
    from xrex.utils.gpu import GpuArch, gpu_arch

    return BLACKWELL_CONFIG if gpu_arch() in (GpuArch.GB200, GpuArch.GB300) else HOPPER_CONFIG


PADDING_SEGMENT_ID = 0
PALLAS_ATTN_RESIDUALS_NAME = "pallas_attn_outputs"


def tanh(x):
    return 2 * jax.lax.logistic(2 * x) - 1


def _gather_block_segments(
    segment_ids: jnp.ndarray,
    cu_seqlens: jnp.ndarray,
    max_seqlen: int,
    block_size: int,
) -> tuple[jnp.ndarray, jnp.ndarray]:
    seqlens = cu_seqlens[1:] - cu_seqlens[:-1]
    num_blocks = pl.cdiv(max_seqlen, block_size)
    block_offsets = jnp.arange(num_blocks) * block_size
    elem_offsets = jnp.arange(block_size)
    positions = block_offsets[:, None] + elem_offsets[None, :]
    idx = cu_seqlens[:-1, None, None] + positions[None, :, :]
    idx = idx.astype(jnp.int32)
    block_segments = jnp.take(segment_ids, idx, mode="clip")
    valid = positions[None, :, :] < seqlens[:, None, None]
    block_segments = jnp.where(valid, block_segments, PADDING_SEGMENT_ID)
    return block_segments, seqlens


def compute_block_state_metadata(
    segment_ids: jnp.ndarray,
    cu_seqlens: jnp.ndarray,
    max_seqlen: int,
    block_size: int,
) -> jnp.ndarray:
    block_segments, _ = _gather_block_segments(segment_ids, cu_seqlens, max_seqlen, block_size)
    has_history = jnp.any(block_segments > 0, axis=-1)
    has_candidate = jnp.any(block_segments < 0, axis=-1)
    has_padding = jnp.any(block_segments == PADDING_SEGMENT_ID, axis=-1)

    state_codes = (
        (has_padding.astype(jnp.int8) << 2)
        | (has_candidate.astype(jnp.int8) << 1)
        | has_history.astype(jnp.int8)
    )
    return state_codes


def compute_last_history_block_idx(
    segment_ids: jnp.ndarray,
    cu_seqlens: jnp.ndarray,
    max_seqlen: int,
    block_size: int,
) -> jnp.ndarray:
    block_segments, _ = _gather_block_segments(segment_ids, cu_seqlens, max_seqlen, block_size)
    num_blocks = block_segments.shape[1]
    has_history_tokens = jnp.any(block_segments > 0, axis=-1)
    last_history_block_idx = jnp.max(
        jnp.where(has_history_tokens, jnp.arange(num_blocks), -1), axis=-1
    )
    return last_history_block_idx[:, None]


def pack_inputs(
    q: jnp.ndarray,
    k: jnp.ndarray,
    v: jnp.ndarray,
    temp: jnp.ndarray,
    segment_ids: jnp.ndarray,
    fixed_total_len: int | None = None,
) -> tuple[jax.Array, jax.Array, jax.Array, jax.Array, jax.Array, jax.Array, jax.Array, int, int]:
    batch_size, seq_len, num_q_heads, head_dim = q.shape
    num_kv_heads = k.shape[2]
    total_len = batch_size * seq_len

    flat_segment_ids = segment_ids.reshape(total_len)
    valid = flat_segment_ids != PADDING_SEGMENT_ID

    flat_q = q.reshape(total_len, num_q_heads, head_dim)
    flat_k = k.reshape(total_len, num_kv_heads, head_dim)
    flat_v = v.reshape(total_len, num_kv_heads, head_dim)
    flat_temp = temp.reshape(total_len)

    q_packed = flat_q[valid]
    k_packed = flat_k[valid]
    v_packed = flat_v[valid]
    temp_packed = flat_temp[valid]
    segment_ids_packed = flat_segment_ids[valid]

    packed_len = int(q_packed.shape[0])
    if fixed_total_len is not None:
        if fixed_total_len < packed_len:
            raise ValueError(
                f"fixed_total_len={fixed_total_len} is smaller than packed length {packed_len}"
            )
        pad_len = fixed_total_len - packed_len
        if pad_len > 0:
            q_packed = jnp.pad(q_packed, ((0, pad_len), (0, 0), (0, 0)))
            k_packed = jnp.pad(k_packed, ((0, pad_len), (0, 0), (0, 0)))
            v_packed = jnp.pad(v_packed, ((0, pad_len), (0, 0), (0, 0)))
            temp_packed = jnp.pad(temp_packed, ((0, pad_len),), constant_values=0)
            segment_ids_packed = jnp.pad(
                segment_ids_packed, ((0, pad_len),), constant_values=PADDING_SEGMENT_ID
            )

    seqlens = jnp.sum(segment_ids != PADDING_SEGMENT_ID, axis=1).astype(jnp.int32)
    cu_seqlens = jnp.concatenate([jnp.array([0], dtype=seqlens.dtype), jnp.cumsum(seqlens)])
    max_seqlen = int(jnp.max(seqlens))

    return (
        q_packed,
        k_packed,
        v_packed,
        temp_packed,
        segment_ids_packed,
        cu_seqlens,
        cu_seqlens,
        max_seqlen,
        max_seqlen,
    )


def mha_forward_kernel(
    q_ref,
    k_ref,
    v_ref,
    temp_ref,
    segment_ref,
    cu_seqlens_q_ref,
    cu_seqlens_k_ref,
    block_state_metadata_ref,
    last_history_k_block_idx_ref,
    o_ref,
    *residual_refs,
    sm_scale: float,
    cap: float,
    cap_method: str,
    causal: bool,
    window_len: int,
    block_q: int,
    block_d: int,
    block_k: int,
):
    del window_len
    start_q = pl.program_id(0)
    batch_id = pl.program_id(1)

    m_i = jnp.full(block_q, -float("inf"), dtype=jnp.float32)
    l_i = jnp.zeros(block_q, dtype=jnp.float32)
    acc = jnp.zeros((block_q, block_d), dtype=jnp.float32)

    q_start = pl.load(cu_seqlens_q_ref, (batch_id,))
    q_end = pl.load(cu_seqlens_q_ref, (batch_id + 1,))
    k_start = pl.load(cu_seqlens_k_ref, (batch_id,))
    k_end = pl.load(cu_seqlens_k_ref, (batch_id + 1,))
    seqlen_q = q_end - q_start
    seqlen_k = k_end - k_start

    offset_q_rel = start_q * block_q
    offset_q = q_start + offset_q_rel

    span_q = offset_q_rel + jnp.arange(block_q)
    q_mask = span_q < seqlen_q

    q = pl.load(
        q_ref,
        (pl.dslice(offset_q, block_q), pl.dslice(None)),
        mask=q_mask[:, None],
        other=0.0,
    )

    seg_q = pl.load(
        segment_ref,
        (pl.dslice(offset_q, block_q),),
        mask=q_mask,
        other=PADDING_SEGMENT_ID,
    ).astype(jnp.int8)[:, None]
    seq_q_is_not_padding = jnp.logical_and(seg_q != PADDING_SEGMENT_ID, q_mask[:, None])

    def body(start_k, carry):
        acc, m_prev, l_prev = carry

        offset_k_rel = start_k * block_k
        offset_k = k_start + offset_k_rel
        span_k = offset_k_rel + jnp.arange(block_k)
        k_mask = span_k < seqlen_k

        k = pl.load(
            k_ref,
            (pl.dslice(offset_k, block_k), pl.dslice(None)),
            mask=k_mask[:, None],
            other=0.0,
        )
        seg_k = pl.load(
            segment_ref,
            (pl.dslice(offset_k, block_k),),
            mask=k_mask,
            other=PADDING_SEGMENT_ID,
        ).astype(jnp.int8)[None, :]

        qk = pl.dot(q, k, trans_b=True)
        if sm_scale != 1.0:
            qk *= sm_scale

        if cap > 0.0:
            if cap_method == "tanh":
                qk = cap * tanh(qk / cap)
            elif cap_method == "soft_sign":
                qk = qk / (1.0 + jnp.abs(qk) / cap)
            else:
                raise ValueError(f"cap_method must be in [tanh, soft_sign], got {cap_method}")

        mask = jnp.logical_or(seg_k > 0, span_q[:, None] == span_k[None, :])
        mask = jnp.logical_and(seq_q_is_not_padding, mask)
        mask = jnp.logical_and(mask, k_mask[None, :])

        if causal:
            causal_mask = span_q[:, None] >= span_k[None, :]
            mask = jnp.logical_and(mask, causal_mask)

        qk = jnp.where(mask, qk, DEFAULT_MASK_VALUE)
        max_logit = jnp.max(qk, axis=1)
        max_logit = jnp.maximum(max_logit, 1.0)
        m_curr = jnp.maximum(m_prev, max_logit)
        p = jnp.exp(qk - m_curr[:, None])
        alpha = jnp.exp(m_prev - m_curr)
        l_prev = l_prev * alpha + jnp.sum(p, axis=1)

        p = p.astype(q.dtype)
        v = pl.load(
            v_ref,
            (pl.dslice(offset_k, block_k), pl.dslice(None)),
            mask=k_mask[:, None],
            other=0.0,
        )
        acc = acc * alpha[:, None] + pl.dot(p, v)
        return acc, m_curr, l_prev

    q_block_state_metadata = jnp.squeeze(pl.load(block_state_metadata_ref, (start_q,)))

    last_history_k_block_idx = jnp.squeeze(pl.load(last_history_k_block_idx_ref, ()))
    num_k_blocks = lax.div(seqlen_k + block_k - 1, block_k)
    history_upper_bound = jnp.where(
        q_block_state_metadata == 4,
        0,
        last_history_k_block_idx + 1,
    )
    history_upper_bound = jnp.minimum(history_upper_bound, num_k_blocks)

    acc, m_i, l_i = lax.fori_loop(0, history_upper_bound, body, (acc, m_i, l_i))

    candidate_lower_bound = offset_q_rel // block_k
    candidate_lower_bound = jnp.maximum(history_upper_bound, candidate_lower_bound)

    candidate_upper_bound = (offset_q_rel + block_q - 1) // block_k + 1
    candidate_upper_bound = jnp.where(
        (q_block_state_metadata & 2) == 2,
        candidate_upper_bound,
        0,
    )
    candidate_upper_bound = jnp.minimum(candidate_upper_bound, num_k_blocks)

    acc, m_i, l_i = lax.fori_loop(
        candidate_lower_bound, candidate_upper_bound, body, (acc, m_i, l_i)
    )

    denom = l_i + (l_i == 0).astype(l_i.dtype)
    acc /= denom[:, None]

    if residual_refs:
        l_ref, m_ref = residual_refs
        pl.store(l_ref, (pl.ds(offset_q, block_q),), l_i, mask=q_mask)
        pl.store(m_ref, (pl.ds(offset_q, block_q),), m_i, mask=q_mask)
    acc = acc.astype(o_ref.dtype)
    store_mask = jnp.broadcast_to(q_mask[:, None], (block_q, block_d))
    pl.store(
        o_ref,
        (pl.dslice(offset_q, block_q), pl.dslice(None)),
        acc,
        mask=store_mask,
    )


@functools.partial(
    jax.custom_vjp,
    nondiff_argnums=[
        7,
        8,
        9,
        10,
        11,
        12,
        13,
        14,
        15,
        16,
        17,
        18,
        19,
    ],
)
@functools.partial(
    jax.jit,
    static_argnames=[
        "max_seqlen_q",
        "max_seqlen_k",
        "sm_scale",
        "cap",
        "cap_method",
        "causal",
        "window_len",
        "z_loss_weight",
        "config",
        "backward_pass_impl",
        "grid",
        "interpret",
        "debug",
    ],
)
def _mha_v1(
    q,
    k,
    v,
    temp,
    segment_ids,
    cu_seqlens_q,
    cu_seqlens_k,
    max_seqlen_q: int,
    max_seqlen_k: int,
    sm_scale: float = 1.0,
    cap: float = -1.0,
    cap_method: str = "tanh",
    causal: bool = True,
    window_len: int = -1,
    z_loss_weight: float = 0.0,
    config: TuningConfig = HOPPER_CONFIG,
    backward_pass_impl: str = "triton_split",
    grid=None,
    interpret: bool = False,
    debug: bool = False,
):
    block_q_fwd = config.block_q
    block_k_fwd = config.block_kv
    num_warps = config.num_warps
    num_stages = config.num_stages
    total_q, num_q_heads, head_dim = q.shape
    total_k = k.shape[0]
    num_kv_heads = k.shape[1]
    batch_size = cu_seqlens_q.shape[0] - 1

    assert total_q == segment_ids.shape[0], "segment_ids length must match q"
    assert total_k == segment_ids.shape[0], "segment_ids length must match k"
    assert temp.shape[0] == total_q, "temp length must match q"
    assert num_q_heads % num_kv_heads == 0, (
        f"num_q_heads ({num_q_heads}) must be divisible by num_kv_heads ({num_kv_heads})"
    )
    assert v.shape[1] == num_kv_heads, "k/v head count must match"
    assert k.shape[2] == head_dim and v.shape[2] == head_dim, "head dim must match"
    assert v.shape[0] == total_k, "k/v total length must match"
    q_heads_per_kv_head = num_q_heads // num_kv_heads

    block_q_fwd = min(block_q_fwd, max_seqlen_q)
    block_k_fwd = min(block_k_fwd, max_seqlen_k)

    num_q_blocks = pl.cdiv(max_seqlen_q, block_q_fwd)
    grid_ = grid
    if grid_ is None:
        grid_ = (num_q_blocks, batch_size, num_q_heads)

    num_warps_ = num_warps
    if num_warps_ is None:
        num_warps_ = 4 if head_dim <= 64 else 8

    q_block_state_metadata = compute_block_state_metadata(
        segment_ids, cu_seqlens_q, max_seqlen_q, block_q_fwd
    )
    last_history_k_block_idx = compute_last_history_block_idx(
        segment_ids, cu_seqlens_k, max_seqlen_k, block_k_fwd
    )

    kernel = functools.partial(
        mha_forward_kernel,
        sm_scale=sm_scale,
        cap=cap,
        cap_method=cap_method,
        block_q=block_q_fwd,
        block_k=block_k_fwd,
        block_d=head_dim,
        causal=causal,
        window_len=window_len,
    )
    out_shape = jax.ShapeDtypeStruct(shape=q.shape, dtype=q.dtype)
    out = pl.pallas_call(
        kernel,
        grid=grid_,
        in_specs=[
            pl.BlockSpec(
                index_map=lambda _, __, k: (0, k, 0),
                block_shape=(total_q, None, head_dim),
            ),
            pl.BlockSpec(
                index_map=lambda _, __, k: (0, k // q_heads_per_kv_head, 0),
                block_shape=(total_k, None, head_dim),
            ),
            pl.BlockSpec(
                index_map=lambda _, __, k: (0, k // q_heads_per_kv_head, 0),
                block_shape=(total_k, None, head_dim),
            ),
            pl.BlockSpec(
                index_map=lambda _, __, k: (0,),
                block_shape=(total_q,),
            ),
            pl.BlockSpec(
                index_map=lambda _, __, k: (0,),
                block_shape=(segment_ids.shape[0],),
            ),
            pl.BlockSpec(
                index_map=lambda _, __, k: (0,),
                block_shape=(cu_seqlens_q.shape[0],),
            ),
            pl.BlockSpec(
                index_map=lambda _, __, k: (0,),
                block_shape=(cu_seqlens_k.shape[0],),
            ),
            pl.BlockSpec(
                index_map=lambda _, j, k: (j, 0),
                block_shape=(None, num_q_blocks),
            ),
            pl.BlockSpec(index_map=lambda _, j, k: (j, 0), block_shape=(None, 1)),
        ],
        out_specs=pl.BlockSpec(
            index_map=lambda _, __, k: (0, k, 0),
            block_shape=(total_q, None, head_dim),
        ),
        compiler_params=CompilerParams(num_warps=num_warps_, num_stages=num_stages),
        out_shape=out_shape,
        debug=debug,
        interpret=interpret,
        name="mha_forward_varlen",
    )(
        q,
        k,
        v,
        temp,
        segment_ids,
        cu_seqlens_q,
        cu_seqlens_k,
        q_block_state_metadata,
        last_history_k_block_idx,
    )
    valid_q = jnp.arange(total_q, dtype=jnp.int32) < cu_seqlens_q[-1]
    return jnp.where(valid_q[:, None, None], out, 0)


def _mha_forward(
    q,
    k,
    v,
    temp,
    segment_ids,
    cu_seqlens_q,
    cu_seqlens_k,
    max_seqlen_q: int,
    max_seqlen_k: int,
    sm_scale: float,
    cap: float,
    cap_method: str,
    causal: bool,
    window_len: int,
    z_loss_weight: float,
    config: TuningConfig,
    backward_pass_impl: str,
    grid: Any,
    interpret: bool,
    debug: bool,
):
    del backward_pass_impl
    block_q_fwd = config.block_q
    block_k_fwd = config.block_kv
    num_warps = config.num_warps
    num_stages = config.num_stages
    total_q, num_q_heads, head_dim = q.shape
    total_k = k.shape[0]
    num_kv_heads = k.shape[1]
    batch_size = cu_seqlens_q.shape[0] - 1
    q_heads_per_kv_head = num_q_heads // num_kv_heads
    used_q = cu_seqlens_q[-1]
    used_k = cu_seqlens_k[-1]
    valid_q = jnp.arange(total_q, dtype=jnp.int32) < used_q
    valid_k = jnp.arange(total_k, dtype=jnp.int32) < used_k

    q = jnp.where(valid_q[:, None, None], q, 0)
    k = jnp.where(valid_k[:, None, None], k, 0)
    v = jnp.where(valid_k[:, None, None], v, 0)
    temp = jnp.where(valid_q, temp, 0)
    segment_ids = jnp.where(valid_q, segment_ids, 0)

    block_q_fwd = min(block_q_fwd, max_seqlen_q)
    block_k_fwd = min(block_k_fwd, max_seqlen_k)
    num_q_blocks = pl.cdiv(max_seqlen_q, block_q_fwd)
    grid_ = grid
    if grid_ is None:
        grid_ = (num_q_blocks, batch_size, num_q_heads)

    num_warps_ = num_warps
    if num_warps_ is None:
        num_warps_ = 4 if head_dim <= 64 else 8

    q_block_state_metadata = compute_block_state_metadata(
        segment_ids, cu_seqlens_q, max_seqlen_q, block_q_fwd
    )
    last_history_k_block_idx = compute_last_history_block_idx(
        segment_ids, cu_seqlens_k, max_seqlen_k, block_k_fwd
    )

    kernel = functools.partial(
        mha_forward_kernel,
        sm_scale=sm_scale,
        cap=cap,
        cap_method=cap_method,
        causal=causal,
        window_len=window_len,
        block_q=block_q_fwd,
        block_k=block_k_fwd,
        block_d=head_dim,
    )
    out, l, m = pl.pallas_call(
        kernel,
        grid=grid_,
        in_specs=[
            pl.BlockSpec(
                index_map=lambda _, __, k: (0, k, 0),
                block_shape=(total_q, None, head_dim),
            ),
            pl.BlockSpec(
                index_map=lambda _, __, k: (0, k // q_heads_per_kv_head, 0),
                block_shape=(total_k, None, head_dim),
            ),
            pl.BlockSpec(
                index_map=lambda _, __, k: (0, k // q_heads_per_kv_head, 0),
                block_shape=(total_k, None, head_dim),
            ),
            pl.BlockSpec(
                index_map=lambda _, __, k: (0,),
                block_shape=(total_q,),
            ),
            pl.BlockSpec(
                index_map=lambda _, __, k: (0,),
                block_shape=(segment_ids.shape[0],),
            ),
            pl.BlockSpec(
                index_map=lambda _, __, k: (0,),
                block_shape=(cu_seqlens_q.shape[0],),
            ),
            pl.BlockSpec(
                index_map=lambda _, __, k: (0,),
                block_shape=(cu_seqlens_k.shape[0],),
            ),
            pl.BlockSpec(index_map=lambda _, j, k: (j, 0), block_shape=(None, num_q_blocks)),
            pl.BlockSpec(index_map=lambda _, j, k: (j, 0), block_shape=(None, 1)),
        ],
        out_specs=[
            pl.BlockSpec(
                index_map=lambda _, __, k: (0, k, 0),
                block_shape=(total_q, None, head_dim),
            ),
            pl.BlockSpec(index_map=lambda _, __, k: (0, k), block_shape=(total_q, None)),
            pl.BlockSpec(index_map=lambda _, __, k: (0, k), block_shape=(total_q, None)),
        ],
        compiler_params=CompilerParams(num_warps=num_warps_, num_stages=num_stages),
        out_shape=[
            jax.ShapeDtypeStruct(shape=q.shape, dtype=q.dtype),
            jax.ShapeDtypeStruct(shape=(total_q, num_q_heads), dtype=jnp.float32),
            jax.ShapeDtypeStruct(shape=(total_q, num_q_heads), dtype=jnp.float32),
        ],
        debug=debug,
        interpret=interpret,
        name="mha_forward",
    )(
        q,
        k,
        v,
        temp,
        segment_ids,
        cu_seqlens_q,
        cu_seqlens_k,
        q_block_state_metadata,
        last_history_k_block_idx,
    )
    out = jnp.where(valid_q[:, None, None], out, 0)
    l = jnp.where(valid_q[:, None], l, 0)
    m = jnp.where(valid_q[:, None], m, 0)
    return out, (
        q,
        k,
        v,
        temp,
        segment_ids,
        cu_seqlens_q,
        cu_seqlens_k,
        checkpoint_name(out, PALLAS_ATTN_RESIDUALS_NAME),
        checkpoint_name(l, PALLAS_ATTN_RESIDUALS_NAME),
        checkpoint_name(m, PALLAS_ATTN_RESIDUALS_NAME),
    )


def _preprocess_backward_packed(
    out: jnp.ndarray,
    do: jnp.ndarray,
    l: jnp.ndarray,
) -> tuple[jnp.ndarray, jnp.ndarray]:
    denom = l.astype(jnp.float32)
    safe_denom = jnp.where(denom == 0, 1.0, denom)
    do_scaled = do.astype(jnp.float32) / safe_denom[:, :, None]
    do_scaled = jnp.where(denom[:, :, None] == 0, 0.0, do_scaled)
    delta = jnp.sum(out.astype(jnp.float32) * do_scaled, axis=-1)
    return do_scaled.astype(do.dtype), delta.astype(jnp.float32)


def mha_backward_kernel_dq(
    q_ref,
    k_ref,
    v_ref,
    temp_ref,
    segment_ref,
    cu_seqlens_q_ref,
    cu_seqlens_k_ref,
    q_block_state_metadata_ref,
    last_history_k_block_idx_ref,
    out_ref,
    do_scaled_ref,
    l_ref,
    m_ref,
    delta_ref,
    dq_ref,
    *,
    sm_scale: float,
    cap: float,
    cap_method: str,
    causal: bool,
    window_len: int,
    z_loss_weight: float,
    block_q: int,
    block_d: int,
    block_k: int,
):
    del out_ref, window_len
    batch_id = pl.program_id(0)
    start_q = pl.program_id(2)

    q_start = pl.load(cu_seqlens_q_ref, (batch_id,))
    q_end = pl.load(cu_seqlens_q_ref, (batch_id + 1,))
    k_start = pl.load(cu_seqlens_k_ref, (batch_id,))
    k_end = pl.load(cu_seqlens_k_ref, (batch_id + 1,))
    seqlen_q = q_end - q_start
    seqlen_k = k_end - k_start

    offset_q_rel = start_q * block_q
    offset_q = q_start + offset_q_rel

    span_q = offset_q_rel + jnp.arange(block_q)
    q_mask = span_q < seqlen_q

    q = pl.load(
        q_ref,
        (pl.ds(offset_q, block_q), slice(None)),
        mask=q_mask[:, None],
        other=0.0,
    )
    m = pl.load(m_ref, (pl.ds(offset_q, block_q),), mask=q_mask, other=0.0)
    l = pl.load(l_ref, (pl.ds(offset_q, block_q),), mask=q_mask, other=0.0)
    do = pl.load(
        do_scaled_ref,
        (pl.ds(offset_q, block_q), slice(None)),
        mask=q_mask[:, None],
        other=0.0,
    )
    di = pl.load(delta_ref, (pl.ds(offset_q, block_q),), mask=q_mask, other=0.0)
    dq = jnp.zeros([block_q, block_d], dtype=jnp.float32)

    seg_q = pl.load(
        segment_ref,
        (pl.ds(offset_q, block_q),),
        mask=q_mask,
        other=PADDING_SEGMENT_ID,
    ).astype(jnp.int8)[:, None]
    seq_q_is_not_padding = jnp.logical_and(seg_q != PADDING_SEGMENT_ID, q_mask[:, None])

    def inner_loop(start_k, dq):
        offset_k_rel = start_k * block_k
        offset_k = k_start + offset_k_rel
        span_k = offset_k_rel + jnp.arange(block_k)
        k_mask = span_k < seqlen_k

        k = pl.load(
            k_ref,
            (pl.ds(offset_k, block_k), slice(None)),
            mask=k_mask[:, None],
            other=0.0,
        )
        v = pl.load(
            v_ref,
            (pl.ds(offset_k, block_k), slice(None)),
            mask=k_mask[:, None],
            other=0.0,
        )
        seg_k = pl.load(
            segment_ref,
            (pl.dslice(offset_k, block_k),),
            mask=k_mask,
            other=PADDING_SEGMENT_ID,
        ).astype(jnp.int8)[None, :]
        qk = pl.dot(q, k, trans_b=True)

        if sm_scale != 1.0:
            qk *= sm_scale

        if cap > 0.0:
            if cap_method == "tanh":
                qk_tanh = tanh(qk / cap)
                qk = cap * qk_tanh
            elif cap_method == "soft_sign":
                soft_sign = 1.0 / (1.0 + jnp.abs(qk) / cap)
                qk = qk * soft_sign
            else:
                raise ValueError("cap_method must be in [tanh, soft_sign]")

        mask = jnp.logical_or(seg_k > 0, span_q[:, None] == span_k[None, :])
        mask = jnp.logical_and(seq_q_is_not_padding, mask)
        mask = jnp.logical_and(mask, k_mask[None, :])
        if causal:
            causal_mask = span_q[:, None] >= span_k[None, :]
            mask = jnp.logical_and(mask, causal_mask)

        p = jnp.exp(qk - m[:, None])
        p = jnp.where(mask, p, 0.0)
        dp = pl.dot(do, v, trans_b=True).astype(jnp.float32) - di[:, None]
        ds = p * dp

        if z_loss_weight > 0:
            ds += z_loss_weight * p * ((jnp.log(l + 1e-12) + m) / l)[:, None]

        if cap > 0.0:
            if cap_method == "tanh":
                ds = ds * (1 - qk_tanh**2)
            elif cap_method == "soft_sign":
                ds = ds * soft_sign**2
            else:
                raise ValueError("cap_method must be in [tanh, soft_sign]")

        if sm_scale != 1.0:
            ds = ds * sm_scale

        dq = dq + pl.dot(ds.astype(k.dtype), k).astype(dq.dtype)
        return dq

    q_block_state_metadata = pl.load(q_block_state_metadata_ref, (start_q,))
    last_history_k_block_idx = pl.load(last_history_k_block_idx_ref, ())
    num_k_blocks = lax.div(seqlen_k + block_k - 1, block_k)

    history_upper_bound = jnp.squeeze(last_history_k_block_idx) + 1
    history_upper_bound = jnp.where(
        q_block_state_metadata == 4,
        0,
        history_upper_bound,
    )
    history_upper_bound = jnp.minimum(history_upper_bound, num_k_blocks)
    dq = lax.fori_loop(0, history_upper_bound, inner_loop, dq)

    candidate_lower_bound = lax.div(offset_q_rel, block_k)
    candidate_lower_bound = jnp.maximum(history_upper_bound, candidate_lower_bound)
    candidate_upper_bound = lax.div(offset_q_rel + block_q - 1, block_k) + 1
    candidate_upper_bound = jnp.where(
        (q_block_state_metadata & 2) == 2,
        candidate_upper_bound,
        0,
    )
    candidate_upper_bound = jnp.minimum(candidate_upper_bound, num_k_blocks)
    dq = lax.fori_loop(candidate_lower_bound, candidate_upper_bound, inner_loop, dq)

    dq_store_mask = jnp.broadcast_to(q_mask[:, None], (block_q, block_d))
    pl.store(
        dq_ref,
        (pl.ds(offset_q, block_q), slice(None)),
        dq.astype(q_ref.dtype),
        mask=dq_store_mask,
    )


def mha_backward_kernel_dkv(
    q_ref,
    k_ref,
    v_ref,
    temp_ref,
    segment_ref,
    cu_seqlens_q_ref,
    cu_seqlens_k_ref,
    block_state_metadata_ref,
    out_ref,
    do_scaled_ref,
    l_ref,
    m_ref,
    delta_ref,
    dk_ref,
    dv_ref,
    *,
    sm_scale: float,
    cap: float,
    cap_method: str,
    causal: bool,
    window_len: int,
    z_loss_weight: float,
    block_q: int,
    block_d: int,
    block_k: int,
):
    del out_ref, window_len
    batch_id = pl.program_id(0)
    start_k = pl.program_id(2)

    q_start = pl.load(cu_seqlens_q_ref, (batch_id,))
    q_end = pl.load(cu_seqlens_q_ref, (batch_id + 1,))
    k_start = pl.load(cu_seqlens_k_ref, (batch_id,))
    k_end = pl.load(cu_seqlens_k_ref, (batch_id + 1,))
    seqlen_q = q_end - q_start
    seqlen_k = k_end - k_start

    offset_k_rel = start_k * block_k
    offset_k = k_start + offset_k_rel

    dv = jnp.zeros([block_k, block_d], dtype=jnp.float32)
    dk = jnp.zeros([block_k, block_d], dtype=jnp.float32)
    span_k = offset_k_rel + jnp.arange(block_k)
    k_mask = span_k < seqlen_k

    k = pl.load(
        k_ref,
        (pl.ds(offset_k, block_k), slice(None)),
        mask=k_mask[:, None],
        other=0.0,
    )
    v = pl.load(
        v_ref,
        (pl.ds(offset_k, block_k), slice(None)),
        mask=k_mask[:, None],
        other=0.0,
    )

    seg_k = pl.load(
        segment_ref,
        (pl.ds(offset_k, block_k),),
        mask=k_mask,
        other=PADDING_SEGMENT_ID,
    ).astype(jnp.int8)[None, :]
    seg_k_is_history = seg_k > 0

    def inner_loop(start_q, carry):
        dv, dk = carry
        offset_q_rel = start_q * block_q
        offset_q = q_start + offset_q_rel
        span_q = offset_q_rel + jnp.arange(block_q)
        q_mask = span_q < seqlen_q

        q = pl.load(
            q_ref,
            (pl.ds(offset_q, block_q), slice(None)),
            mask=q_mask[:, None],
            other=0.0,
        )
        qk = pl.dot(q, k, trans_b=True)
        seg_q = pl.load(
            segment_ref,
            (pl.ds(offset_q, block_q),),
            mask=q_mask,
            other=PADDING_SEGMENT_ID,
        ).astype(jnp.int8)[:, None]
        seq_q_is_not_padding = jnp.logical_and(seg_q != PADDING_SEGMENT_ID, q_mask[:, None])

        m = pl.load(m_ref, (pl.ds(offset_q, block_q),), mask=q_mask, other=0.0)
        do = pl.load(
            do_scaled_ref,
            (pl.ds(offset_q, block_q), slice(None)),
            mask=q_mask[:, None],
            other=0.0,
        )
        di = pl.load(delta_ref, (pl.ds(offset_q, block_q),), mask=q_mask, other=0.0)

        if sm_scale != 1.0:
            qk *= sm_scale

        if cap > 0.0:
            if cap_method == "tanh":
                qk_tanh = tanh(qk / cap)
                qk = cap * qk_tanh
            elif cap_method == "soft_sign":
                soft_sign = 1.0 / (1.0 + jnp.abs(qk) / cap)
                qk = qk * soft_sign
            else:
                raise ValueError("cap_method must be in [tanh, soft_sign]")

        mask = jnp.logical_or(seg_k_is_history, span_q[:, None] == span_k[None, :])
        mask = jnp.logical_and(seq_q_is_not_padding, mask)
        mask = jnp.logical_and(mask, k_mask[None, :])
        if causal:
            causal_mask = span_q[:, None] >= span_k[None, :]
            mask = jnp.logical_and(mask, causal_mask)

        p = jnp.exp(qk - m[:, None])
        p = jnp.where(mask, p, 0.0)
        dot_result = pl.dot(p.astype(do.dtype), do, trans_a=True)
        dv = dv + dot_result.astype(dv.dtype)

        dp = pl.dot(do, v, trans_b=True) - di[:, None]

        ds = p * dp
        if z_loss_weight > 0:
            l = pl.load(l_ref, (pl.ds(offset_q, block_q),), mask=q_mask, other=0.0)
            ds += z_loss_weight * p * ((jnp.log(l + 1e-12) + m) / l)[:, None]
        if cap > 0.0:
            if cap_method == "tanh":
                ds = ds * (1 - qk_tanh**2)
            elif cap_method == "soft_sign":
                ds = ds * soft_sign**2
            else:
                raise ValueError("cap_method must be in [tanh, soft_sign]")
        if sm_scale != 1.0:
            ds = ds * sm_scale
        dk = dk + pl.dot(ds.astype(q_ref.dtype), q, trans_a=True).astype(dk.dtype)
        return dv, dk

    k_block_state_metadata = pl.load(block_state_metadata_ref, (start_k,))
    num_q_blocks = lax.div(seqlen_q + block_q - 1, block_q)

    lower_bound = jnp.where(
        (k_block_state_metadata & 1) == 1,
        0,
        offset_k_rel // block_q,
    )

    upper_bound = jnp.where(
        (k_block_state_metadata & 1) == 1,
        num_q_blocks,
        (offset_k_rel + block_k - 1) // block_q + 1,
    )
    upper_bound = jnp.minimum(upper_bound, num_q_blocks)
    dv, dk = lax.fori_loop(lower_bound, upper_bound, inner_loop, (dv, dk))

    kv_store_mask = jnp.broadcast_to(k_mask[:, None], (block_k, block_d))
    pl.store(
        dv_ref,
        (pl.ds(offset_k, block_k), slice(None)),
        dv.astype(dv_ref.dtype),
        mask=kv_store_mask,
    )
    pl.store(
        dk_ref,
        (pl.ds(offset_k, block_k), slice(None)),
        dk.astype(dk_ref.dtype),
        mask=kv_store_mask,
    )


def _mha_backward(
    max_seqlen_q: int,
    max_seqlen_k: int,
    sm_scale: float,
    cap: float,
    cap_method: str,
    causal: bool,
    window_len: int,
    z_loss_weight: float,
    config: TuningConfig,
    backward_pass_impl: str,
    grid: Any,
    interpret: bool,
    debug: bool,
    res,
    do,
):
    del grid
    assert config.block_q_dq is not None
    assert config.block_kv_dq is not None
    assert config.block_q_dkv is not None
    assert config.block_kv_dkv is not None
    assert config.bwd_num_warps is not None
    assert config.bwd_num_stages is not None

    block_q_bwd_dq = config.block_q_dq
    block_k_bwd_dq = config.block_kv_dq
    block_q_bwd_dkv = config.block_q_dkv
    block_k_bwd_dkv = config.block_kv_dkv
    q, k, v, temp, segment_ids, cu_seqlens_q, cu_seqlens_k, out, l, m = res

    total_q, num_q_heads, head_dim = q.shape
    total_k = k.shape[0]
    num_kv_heads = k.shape[1]
    q_heads_per_kv_head = num_q_heads // num_kv_heads
    batch_size = cu_seqlens_q.shape[0] - 1
    used_q = cu_seqlens_q[-1]
    used_k = cu_seqlens_k[-1]
    valid_q = jnp.arange(total_q, dtype=jnp.int32) < used_q
    valid_k = jnp.arange(total_k, dtype=jnp.int32) < used_k

    q = jnp.where(valid_q[:, None, None], q, 0)
    k = jnp.where(valid_k[:, None, None], k, 0)
    v = jnp.where(valid_k[:, None, None], v, 0)
    temp = jnp.where(valid_q, temp, 0)
    segment_ids = jnp.where(valid_q, segment_ids, 0)
    out = jnp.where(valid_q[:, None, None], out, 0)
    l = jnp.where(valid_q[:, None], l, 0)
    m = jnp.where(valid_q[:, None], m, 0)
    do = jnp.where(valid_q[:, None, None], do, 0)

    block_q_bwd_dq = min(block_q_bwd_dq, max_seqlen_q)
    block_k_bwd_dq = min(block_k_bwd_dq, max_seqlen_k)
    block_q_bwd_dkv = min(block_q_bwd_dkv, max_seqlen_q)
    block_k_bwd_dkv = min(block_k_bwd_dkv, max_seqlen_k)

    num_q_blocks_dq = pl.cdiv(max_seqlen_q, block_q_bwd_dq)
    num_k_blocks_dkv = pl.cdiv(max_seqlen_k, block_k_bwd_dkv)

    do_scaled, delta = _preprocess_backward_packed(out, do, l)

    if backward_pass_impl == "triton_split":
        dtemp = jnp.zeros(temp.shape, temp.dtype)
        dsegment = jnp.zeros(segment_ids.shape, q.dtype)

        num_warps = config.bwd_num_warps
        num_stages = config.bwd_num_stages

        grid_q = (batch_size, num_q_heads, num_q_blocks_dq)
        q_block_bwd_dq_state_metadata = compute_block_state_metadata(
            segment_ids, cu_seqlens_q, max_seqlen_q, block_q_bwd_dq
        )
        last_history_k_block_bwd_dq_idx = compute_last_history_block_idx(
            segment_ids, cu_seqlens_k, max_seqlen_k, block_k_bwd_dq
        )

        dq = pl.pallas_call(
            functools.partial(
                mha_backward_kernel_dq,
                block_q=block_q_bwd_dq,
                block_d=head_dim,
                block_k=block_k_bwd_dq,
                sm_scale=sm_scale,
                cap=cap,
                cap_method=cap_method,
                causal=causal,
                window_len=window_len,
                z_loss_weight=z_loss_weight,
            ),
            grid=grid_q,
            out_shape=jax.ShapeDtypeStruct(shape=q.shape, dtype=q.dtype),
            in_specs=[
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k, 0),
                    block_shape=(total_q, None, head_dim),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k // q_heads_per_kv_head, 0),
                    block_shape=(total_k, None, head_dim),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k // q_heads_per_kv_head, 0),
                    block_shape=(total_k, None, head_dim),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0,),
                    block_shape=(total_q,),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0,),
                    block_shape=(segment_ids.shape[0],),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0,),
                    block_shape=(cu_seqlens_q.shape[0],),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0,),
                    block_shape=(cu_seqlens_k.shape[0],),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (j, 0),
                    block_shape=(None, num_q_blocks_dq),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (j, 0),
                    block_shape=(None, 1),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k, 0),
                    block_shape=(total_q, None, head_dim),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k, 0),
                    block_shape=(total_q, None, head_dim),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k),
                    block_shape=(total_q, None),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k),
                    block_shape=(total_q, None),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k),
                    block_shape=(total_q, None),
                ),
            ],
            out_specs=pl.BlockSpec(
                index_map=lambda j, k, _: (0, k, 0),
                block_shape=(total_q, None, head_dim),
            ),
            name="mha_backward_q",
            debug=debug,
            interpret=interpret,
            compiler_params=CompilerParams(num_warps=num_warps, num_stages=num_stages),
        )(
            q,
            k,
            v,
            temp,
            segment_ids,
            cu_seqlens_q,
            cu_seqlens_k,
            q_block_bwd_dq_state_metadata,
            last_history_k_block_bwd_dq_idx,
            out,
            do_scaled,
            l,
            m,
            delta,
        )
        dq = jnp.where(valid_q[:, None, None], dq, 0)

        grid_kv = (batch_size, num_q_heads, num_k_blocks_dkv)
        k_block_bwd_dkv_state_metadata = compute_block_state_metadata(
            segment_ids, cu_seqlens_k, max_seqlen_k, block_k_bwd_dkv
        )
        out_shapes_kv = [
            jax.ShapeDtypeStruct((total_k, num_q_heads, head_dim), k.dtype),
            jax.ShapeDtypeStruct((total_k, num_q_heads, head_dim), v.dtype),
        ]
        dk, dv = pl.pallas_call(
            functools.partial(
                mha_backward_kernel_dkv,
                block_q=block_q_bwd_dkv,
                block_d=head_dim,
                block_k=block_k_bwd_dkv,
                sm_scale=sm_scale,
                cap=cap,
                cap_method=cap_method,
                causal=causal,
                window_len=window_len,
                z_loss_weight=z_loss_weight,
            ),
            grid=grid_kv,
            out_shape=out_shapes_kv,
            in_specs=[
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k, 0),
                    block_shape=(total_q, None, head_dim),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k // q_heads_per_kv_head, 0),
                    block_shape=(total_k, None, head_dim),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k // q_heads_per_kv_head, 0),
                    block_shape=(total_k, None, head_dim),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0,),
                    block_shape=(total_q,),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0,),
                    block_shape=(segment_ids.shape[0],),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0,),
                    block_shape=(cu_seqlens_q.shape[0],),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0,),
                    block_shape=(cu_seqlens_k.shape[0],),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (j, 0),
                    block_shape=(None, num_k_blocks_dkv),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k, 0),
                    block_shape=(total_q, None, head_dim),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k, 0),
                    block_shape=(total_q, None, head_dim),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k),
                    block_shape=(total_q, None),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k),
                    block_shape=(total_q, None),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k),
                    block_shape=(total_q, None),
                ),
            ],
            out_specs=[
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k, 0),
                    block_shape=(total_k, None, head_dim),
                ),
                pl.BlockSpec(
                    index_map=lambda j, k, _: (0, k, 0),
                    block_shape=(total_k, None, head_dim),
                ),
            ],
            name="mha_backward_kv",
            debug=debug,
            interpret=interpret,
            compiler_params=CompilerParams(num_warps=num_warps, num_stages=num_stages),
        )(
            q,
            k,
            v,
            temp,
            segment_ids,
            cu_seqlens_q,
            cu_seqlens_k,
            k_block_bwd_dkv_state_metadata,
            out,
            do_scaled,
            l,
            m,
            delta,
        )
        if q_heads_per_kv_head > 1:
            dk = dk.reshape(total_k, num_kv_heads, q_heads_per_kv_head, head_dim)
            dk = dk.astype(jnp.float32).sum(axis=2).astype(k.dtype)
            dv = dv.reshape(total_k, num_kv_heads, q_heads_per_kv_head, head_dim)
            dv = dv.astype(jnp.float32).sum(axis=2).astype(v.dtype)
        dk = jnp.where(valid_k[:, None, None], dk, 0)
        dv = jnp.where(valid_k[:, None, None], dv, 0)
    else:
        raise ValueError(f"Invalid backward pass implementation: {backward_pass_impl}")

    return dq.astype(q.dtype), dk, dv, dtemp, dsegment, None, None


_mha_v1.defvjp(_mha_forward, _mha_backward)


_LOG2E = 1.4426950408889634


@dataclasses.dataclass(frozen=True)
class V2TuningConfig:
    block_q: int = 128
    block_kv: int = 32
    num_warps: int = 4
    num_stages: int = 3
    block_q_dq: int = 64
    block_kv_dq: int = 64
    dq_num_warps: int = 4
    dq_num_stages: int = 2
    block_q_dkv: int = 64
    block_kv_dkv: int = 64
    dkv_num_warps: int = 4
    dkv_num_stages: int = 2


HOPPER_V2_CONFIG = V2TuningConfig()


@functools.cache
def _is_hopper() -> bool:
    try:
        from xrex.utils.gpu import GpuArch, gpu_arch

        return jax.default_backend() == "gpu" and gpu_arch() in (GpuArch.H100, GpuArch.H200)
    except Exception:
        return False


@functools.cache
def _v1_config() -> TuningConfig:
    try:
        return get_device_tuning_config()
    except Exception:
        return HOPPER_CONFIG


@functools.cache
def _default_config() -> TuningConfig | V2TuningConfig:
    return HOPPER_V2_CONFIG if _is_hopper() else _v1_config()


def _v2_supported(q, k, cap, cap_method, causal, z_loss_weight, grid, interpret) -> bool:
    return (
        _is_hopper()
        and not causal
        and not interpret
        and grid is None
        and z_loss_weight == 0.0
        and (cap <= 0.0 or cap_method == "soft_sign")
        and q.shape[-1] == 128
        and q.dtype in (jnp.bfloat16, jnp.float16)
        and q.dtype == k.dtype
    )


def _v2_block_flags(segment_ids, cu_seqlens, max_seqlen, block):
    seqlens = cu_seqlens[1:] - cu_seqlens[:-1]
    num_blocks = pl.cdiv(max_seqlen, block)
    pos = jnp.arange(num_blocks * block, dtype=jnp.int32).reshape(num_blocks, block)
    idx = cu_seqlens[:-1, None, None] + pos[None]
    seg = jnp.take(segment_ids, idx, mode="clip")
    seg = jnp.where(pos[None] < seqlens[:, None, None], seg, PADDING_SEGMENT_ID)
    return dict(
        all_hist=jnp.all(seg > 0, axis=-1),
        all_valid=jnp.all(seg != PADDING_SEGMENT_ID, axis=-1),
        has_hist=jnp.any(seg > 0, axis=-1),
        has_cand=jnp.any(seg < 0, axis=-1),
        has_valid=jnp.any(seg != PADDING_SEGMENT_ID, axis=-1),
    )


def _v2_range_and_list(full, member):
    ar = jnp.arange(full.shape[1], dtype=jnp.int32)[None, :]
    hi = jnp.max(jnp.where(full, ar + 1, 0), axis=1)
    lo = jnp.max(jnp.where(~full & (ar < hi[:, None]), ar + 1, 0), axis=1)
    part = member & ~((ar >= lo[:, None]) & (ar < hi[:, None]))
    cnt = jnp.sum(part, axis=1)
    idx = jnp.argsort((~part).astype(jnp.int32), axis=1, stable=True)
    return lo.astype(jnp.int32), hi.astype(jnp.int32), cnt.astype(jnp.int32), idx.astype(jnp.int32)


def _v2_work_table(cu_seqlens, block, max_seqlen, total):
    batch = cu_seqlens.shape[0] - 1
    seqlens = cu_seqlens[1:] - cu_seqlens[:-1]
    nblk = jnp.minimum((seqlens + block - 1) // block, pl.cdiv(max_seqlen, block))
    order = jnp.argsort(-seqlens, stable=True)
    cnt = nblk[order]
    ends = jnp.cumsum(cnt)
    starts = ends - cnt
    num_items = total // block + batch + 1
    w = jnp.arange(num_items, dtype=jnp.int32)
    r = jnp.minimum(jnp.searchsorted(ends, w, side="right", method="compare_all"), batch - 1)
    real = w < ends[-1]
    users = jnp.where(real, order[r], -1 - (w - ends[-1])).astype(jnp.int32)
    blocks = jnp.where(real, w - starts[r], 0).astype(jnp.int32)
    return users, blocks, num_items


def _v2_full_spec(x):
    return pl.BlockSpec(x.shape, lambda *_: (0,) * x.ndim)


def _v2_asm(asm, x):
    return plgpu.elementwise_inline_asm(
        asm,
        args=[x],
        constraints="=f,f",
        pack=1,
        result_shape_dtypes=[jax.ShapeDtypeStruct(x.shape, jnp.float32)],
    )[0]


def _v2_exp2(x):
    return _v2_asm("ex2.approx.ftz.f32 $0, $1;", x)


def _v2_logits(s, sm_scale, cap):
    y = s * (sm_scale * _LOG2E)
    if cap > 0.0:
        r = _v2_asm("rcp.approx.ftz.f32 $0, $1;", 1.0 + jnp.abs(y) * (1.0 / (cap * _LOG2E)))
        return y * r, r
    return y, None


def _v2_zero_tail(ref, head, user, cu_ref, block, total):
    r0 = pl.load(cu_ref, (cu_ref.shape[0] - 1,)) + (-1 - user) * block
    mask = (r0 + jnp.arange(block)) < total
    zeros = jnp.zeros((block, ref.shape[-1]), ref.dtype)
    pl.store(ref, (pl.ds(r0, block), head, slice(None)), zeros, mask=mask[:, None])


def _v2_q_tile_prologue(seg_ref, cuq_ref, cuk_ref, qflag_ref, user, block_idx, head, block_q):
    qs = pl.load(cuq_ref, (user,))
    seqlen_q = pl.load(cuq_ref, (user + 1,)) - qs
    ks = pl.load(cuk_ref, (user,))
    seqlen_k = pl.load(cuk_ref, (user + 1,)) - ks
    q0r = block_idx * block_q
    rows = q0r + jnp.arange(block_q)
    rmask = rows < seqlen_q
    qsl = (pl.ds(qs + q0r, block_q), head, slice(None))
    seg_q = pl.load(seg_ref, (pl.ds(qs + q0r, block_q),), mask=rmask, other=PADDING_SEGMENT_ID)
    flag = pl.load(qflag_ref, (user, block_idx))
    return dict(
        ks=ks,
        seqlen_k=seqlen_k,
        q0r=q0r,
        rows=rows,
        rmask=rmask,
        qsl=qsl,
        vsl=(head, pl.ds(qs + q0r, block_q)),
        rvalid=rmask & (seg_q != PADDING_SEGMENT_ID),
        has_valid=(flag & 1) != 0,
        has_cand=(flag & 2) != 0,
    )


def _v2_kv_loop_parts(
    t, user, kv_head, k_ref, v_ref, seg_ref, kflo_ref, kfhi_ref, kpcnt_ref, block_q, block_k
):
    ks, seqlen_k = t["ks"], t["seqlen_k"]

    def load_full(i):
        k0 = ks + i * block_k
        return (
            pl.load(k_ref, (pl.ds(k0, block_k), kv_head, slice(None))),
            pl.load(v_ref, (pl.ds(k0, block_k), kv_head, slice(None))),
        )

    def load_masked(kb):
        c0r = kb * block_k
        cols = c0r + jnp.arange(block_k)
        cmask = cols < seqlen_k
        sl = (pl.ds(ks + c0r, block_k), kv_head, slice(None))
        kt = pl.load(k_ref, sl, mask=cmask[:, None], other=0.0)
        vt = pl.load(v_ref, sl, mask=cmask[:, None], other=0.0)
        seg_k = pl.load(seg_ref, (pl.ds(ks + c0r, block_k),), mask=cmask, other=PADDING_SEGMENT_ID)
        return cols, kt, vt, seg_k

    lo = pl.load(kflo_ref, (user,))
    hi = jnp.where(t["has_valid"], pl.load(kfhi_ref, (user,)), lo)
    npart = jnp.where(t["has_valid"], pl.load(kpcnt_ref, (user,)), 0)
    dlo = lax.div(t["q0r"], block_k)
    dhi = jnp.minimum(
        lax.div(t["q0r"] + block_q + block_k - 1, block_k), lax.div(seqlen_k + block_k - 1, block_k)
    )
    dhi = jnp.where(t["has_cand"], dhi, dlo)
    return load_full, load_masked, (lo, hi), npart, (dlo, dhi)


def _v2_fwd_kernel(
    wu_ref,
    wj_ref,
    cuq_ref,
    cuk_ref,
    seg_ref,
    kflo_ref,
    kfhi_ref,
    kpcnt_ref,
    kpidx_ref,
    qflag_ref,
    q_ref,
    k_ref,
    v_ref,
    o_ref,
    lse_ref,
    *,
    sm_scale,
    cap,
    block_q,
    block_k,
    q_heads_per_kv_head,
    total,
):
    head = pl.program_id(0)
    w = pl.program_id(1)
    user = pl.load(wu_ref, (w,))
    block_idx = pl.load(wj_ref, (w,))

    @pl.when(user < 0)
    def _():
        _v2_zero_tail(o_ref, head, user, cuq_ref, block_q, total)
        r0 = pl.load(cuq_ref, (cuq_ref.shape[0] - 1,)) + (-1 - user) * block_q
        pl.store(
            lse_ref,
            (head, pl.ds(r0, block_q)),
            jnp.zeros(block_q, jnp.float32),
            mask=r0 + jnp.arange(block_q) < total,
        )

    @pl.when(user >= 0)
    def _():
        t = _v2_q_tile_prologue(
            seg_ref, cuq_ref, cuk_ref, qflag_ref, user, block_idx, head, block_q
        )
        q = pl.load(q_ref, t["qsl"], mask=t["rmask"][:, None], other=0.0)
        load_full, load_masked, full_rng, npart, diag_rng = _v2_kv_loop_parts(
            t,
            user,
            lax.div(head, q_heads_per_kv_head),
            k_ref,
            v_ref,
            seg_ref,
            kflo_ref,
            kfhi_ref,
            kpcnt_ref,
            block_q,
            block_k,
        )

        def update(y, vt, carry):
            acc, m, l = carry
            m_new = jnp.maximum(m, jnp.max(y, axis=1))
            alpha = _v2_exp2(m - m_new)
            p = _v2_exp2(y - m_new[:, None])
            l = l * alpha + jnp.sum(p, axis=1)
            acc = acc * alpha[:, None] + pl.dot(p.astype(vt.dtype), vt)
            return acc, m_new, l

        def body_full(i, carry):
            kt, vt = load_full(i)
            y, _ = _v2_logits(pl.dot(q, kt, trans_b=True), sm_scale, cap)
            return update(y, vt, carry)

        def body_part(i, carry):
            _, kt, vt, seg_k = load_masked(pl.load(kpidx_ref, (user, i)))
            y, _ = _v2_logits(pl.dot(q, kt, trans_b=True), sm_scale, cap)
            return update(jnp.where((seg_k > 0)[None, :], y, DEFAULT_MASK_VALUE), vt, carry)

        def body_diag(kb, carry):
            cols, kt, vt, seg_k = load_masked(kb)
            y, _ = _v2_logits(pl.dot(q, kt, trans_b=True), sm_scale, cap)
            allowed = (t["rows"][:, None] == cols[None, :]) & (seg_k < 0)[None, :]
            return update(jnp.where(allowed, y, DEFAULT_MASK_VALUE), vt, carry)

        carry = (
            jnp.zeros((block_q, q_ref.shape[-1]), jnp.float32),
            jnp.full((block_q,), _LOG2E, jnp.float32),
            jnp.zeros((block_q,), jnp.float32),
        )
        carry = lax.fori_loop(*full_rng, body_full, carry)
        carry = lax.fori_loop(0, npart, body_part, carry)
        acc, m, l = lax.fori_loop(*diag_rng, body_diag, carry)

        ok = t["rvalid"] & (l > 0.0)
        l_safe = jnp.where(ok, l, 1.0)
        out = jnp.where(ok[:, None], acc * (1.0 / l_safe)[:, None], 0.0)
        lse = jnp.where(ok, m + jnp.log(l_safe) * _LOG2E, jnp.inf)
        pl.store(o_ref, t["qsl"], out.astype(o_ref.dtype), mask=t["rmask"][:, None])
        pl.store(lse_ref, t["vsl"], lse, mask=t["rmask"])


def _v2_q_side_meta(segment_ids, cuq, cuk, max_seqlen_q, max_seqlen_k, block_q, block_k, total):
    fk = _v2_block_flags(segment_ids, cuk, max_seqlen_k, block_k)
    kflo, kfhi, kpcnt, kpidx = _v2_range_and_list(fk["all_hist"], fk["has_hist"])
    fq = _v2_block_flags(segment_ids, cuq, max_seqlen_q, block_q)
    qflag = fq["has_valid"].astype(jnp.int32) | (fq["has_cand"].astype(jnp.int32) << 1)
    wu, wj, num_items = _v2_work_table(cuq, block_q, max_seqlen_q, total)
    return (wu, wj, kflo, kfhi, kpcnt, kpidx, qflag), num_items


def _v2_block(block, max_seqlen):
    return min(block, max(16, pl.next_power_of_2(max_seqlen)))


def _v2_forward(
    q, k, v, segment_ids, cuq, cuk, max_seqlen_q, max_seqlen_k, sm_scale, cap, config, debug
):
    total, num_q_heads, head_dim = q.shape
    block_q = _v2_block(config.block_q, max_seqlen_q)
    block_k = _v2_block(config.block_kv, max_seqlen_k)
    meta, num_items = _v2_q_side_meta(
        segment_ids, cuq, cuk, max_seqlen_q, max_seqlen_k, block_q, block_k, total
    )
    args = (meta[0], meta[1], cuq, cuk, segment_ids, *meta[2:], q, k, v)
    return pl.pallas_call(
        functools.partial(
            _v2_fwd_kernel,
            sm_scale=sm_scale,
            cap=cap,
            block_q=block_q,
            block_k=block_k,
            q_heads_per_kv_head=num_q_heads // k.shape[1],
            total=total,
        ),
        grid=(num_q_heads, num_items),
        in_specs=[_v2_full_spec(a) for a in args],
        out_specs=[
            pl.BlockSpec((total, num_q_heads, head_dim), lambda *_: (0, 0, 0)),
            pl.BlockSpec((num_q_heads, total), lambda *_: (0, 0)),
        ],
        out_shape=[
            jax.ShapeDtypeStruct(q.shape, q.dtype),
            jax.ShapeDtypeStruct((num_q_heads, total), jnp.float32),
        ],
        compiler_params=CompilerParams(num_warps=config.num_warps, num_stages=config.num_stages),
        debug=debug,
        name="mha_forward_v2",
    )(*args)


def _v2_dq_kernel(
    wu_ref,
    wj_ref,
    cuq_ref,
    cuk_ref,
    seg_ref,
    kflo_ref,
    kfhi_ref,
    kpcnt_ref,
    kpidx_ref,
    qflag_ref,
    q_ref,
    k_ref,
    v_ref,
    o_ref,
    do_ref,
    lse_ref,
    dq_ref,
    delta_ref,
    *,
    sm_scale,
    cap,
    block_q,
    block_k,
    q_heads_per_kv_head,
    total,
):
    head = pl.program_id(0)
    w = pl.program_id(1)
    user = pl.load(wu_ref, (w,))
    block_idx = pl.load(wj_ref, (w,))

    @pl.when(user < 0)
    def _():
        _v2_zero_tail(dq_ref, head, user, cuq_ref, block_q, total)
        r0 = pl.load(cuq_ref, (cuq_ref.shape[0] - 1,)) + (-1 - user) * block_q
        pl.store(
            delta_ref,
            (head, pl.ds(r0, block_q)),
            jnp.zeros(block_q, jnp.float32),
            mask=r0 + jnp.arange(block_q) < total,
        )

    @pl.when(user >= 0)
    def _():
        t = _v2_q_tile_prologue(
            seg_ref, cuq_ref, cuk_ref, qflag_ref, user, block_idx, head, block_q
        )
        rm = t["rmask"][:, None]
        q = pl.load(q_ref, t["qsl"], mask=rm, other=0.0)
        do = pl.load(do_ref, t["qsl"], mask=rm, other=0.0)
        o = pl.load(o_ref, t["qsl"], mask=rm, other=0.0)
        delta = jnp.sum(o.astype(jnp.float32) * do.astype(jnp.float32), axis=1)
        lse = pl.load(lse_ref, t["vsl"], mask=t["rmask"], other=jnp.inf)
        load_full, load_masked, full_rng, npart, diag_rng = _v2_kv_loop_parts(
            t,
            user,
            lax.div(head, q_heads_per_kv_head),
            k_ref,
            v_ref,
            seg_ref,
            kflo_ref,
            kfhi_ref,
            kpcnt_ref,
            block_q,
            block_k,
        )

        def step(kt, vt, allowed, dq):
            y, r = _v2_logits(pl.dot(q, kt, trans_b=True), sm_scale, cap)
            p = _v2_exp2(y - lse[:, None])
            if allowed is not None:
                p = jnp.where(allowed, p, 0.0)
            ds = p * (pl.dot(do, vt, trans_b=True) - delta[:, None])
            if r is not None:
                ds = ds * (r * r)
            return dq + pl.dot(ds.astype(kt.dtype), kt)

        def body_full(i, dq):
            kt, vt = load_full(i)
            return step(kt, vt, None, dq)

        def body_part(i, dq):
            _, kt, vt, seg_k = load_masked(pl.load(kpidx_ref, (user, i)))
            return step(kt, vt, (seg_k > 0)[None, :], dq)

        def body_diag(kb, dq):
            cols, kt, vt, seg_k = load_masked(kb)
            allowed = (t["rows"][:, None] == cols[None, :]) & (seg_k < 0)[None, :]
            return step(kt, vt, allowed, dq)

        dq = jnp.zeros((block_q, q_ref.shape[-1]), jnp.float32)
        dq = lax.fori_loop(*full_rng, body_full, dq)
        dq = lax.fori_loop(0, npart, body_part, dq)
        dq = lax.fori_loop(*diag_rng, body_diag, dq)
        dq = jnp.where(t["rvalid"][:, None], dq * sm_scale, 0.0)
        pl.store(dq_ref, t["qsl"], dq.astype(dq_ref.dtype), mask=rm)
        pl.store(delta_ref, t["vsl"], jnp.where(t["rvalid"], delta, 0.0), mask=t["rmask"])


def _v2_dkv_kernel(
    wu_ref,
    wj_ref,
    cuq_ref,
    cuk_ref,
    seg_ref,
    qflo_ref,
    qfhi_ref,
    qpcnt_ref,
    qpidx_ref,
    kflag_ref,
    q_ref,
    k_ref,
    v_ref,
    do_ref,
    lse_ref,
    delta_ref,
    dk_ref,
    dv_ref,
    *,
    sm_scale,
    cap,
    block_q,
    block_k,
    q_heads_per_kv_head,
    total,
):
    G = q_heads_per_kv_head
    kv_head = pl.program_id(0)
    w = pl.program_id(1)
    user = pl.load(wu_ref, (w,))
    kb = pl.load(wj_ref, (w,))

    @pl.when(user < 0)
    def _():
        _v2_zero_tail(dk_ref, kv_head, user, cuk_ref, block_k, total)
        _v2_zero_tail(dv_ref, kv_head, user, cuk_ref, block_k, total)

    @pl.when(user >= 0)
    def _():
        qs = pl.load(cuq_ref, (user,))
        seqlen_q = pl.load(cuq_ref, (user + 1,)) - qs
        ks = pl.load(cuk_ref, (user,))
        seqlen_k = pl.load(cuk_ref, (user + 1,)) - ks
        c0r = kb * block_k
        cols = c0r + jnp.arange(block_k)
        cmask = cols < seqlen_k
        ksl = (pl.ds(ks + c0r, block_k), kv_head, slice(None))
        kt = pl.load(k_ref, ksl, mask=cmask[:, None], other=0.0)
        vt = pl.load(v_ref, ksl, mask=cmask[:, None], other=0.0)
        seg_k = pl.load(seg_ref, (pl.ds(ks + c0r, block_k),), mask=cmask, other=PADDING_SEGMENT_ID)
        flag = pl.load(kflag_ref, (user, kb))
        k_full = (flag & 1) != 0
        k_hist = (flag & 2) != 0
        k_cand = (flag & 4) != 0

        def step(i, q_block, rmask, allowed, carry):
            dk, dv = carry
            head = kv_head * G + lax.rem(i, G)
            q0 = qs + q_block * block_q
            qsl = (pl.ds(q0, block_q), head, slice(None))
            vsl = (head, pl.ds(q0, block_q))
            if rmask is None:
                qt = pl.load(q_ref, qsl)
                dot_ = pl.load(do_ref, qsl)
                lse = pl.load(lse_ref, vsl)
                dlt = pl.load(delta_ref, vsl)
            else:
                qt = pl.load(q_ref, qsl, mask=rmask[:, None], other=0.0)
                dot_ = pl.load(do_ref, qsl, mask=rmask[:, None], other=0.0)
                lse = pl.load(lse_ref, vsl, mask=rmask, other=jnp.inf)
                dlt = pl.load(delta_ref, vsl, mask=rmask, other=0.0)
            y, r = _v2_logits(pl.dot(qt, kt, trans_b=True), sm_scale, cap)
            p = _v2_exp2(y - lse[:, None])
            if allowed is not None:
                p = jnp.where(allowed, p, 0.0)
            dv = dv + pl.dot(p.astype(dot_.dtype), dot_, trans_a=True)
            ds = p * (pl.dot(dot_, vt, trans_b=True) - dlt[:, None])
            if r is not None:
                ds = ds * (r * r)
            dk = dk + pl.dot(ds.astype(qt.dtype), qt, trans_a=True)
            return dk, dv

        qflo = pl.load(qflo_ref, (user,))
        nqf = pl.load(qfhi_ref, (user,)) - qflo
        nqp = pl.load(qpcnt_ref, (user,))

        def body_full(i, carry):
            return step(i, qflo + lax.div(i, G), None, None, carry)

        def body_full_kmask(i, carry):
            return step(i, qflo + lax.div(i, G), None, (seg_k > 0)[None, :], carry)

        def q_side(q_block):
            rows = q_block * block_q + jnp.arange(block_q)
            rmask = rows < seqlen_q
            seg_q = pl.load(
                seg_ref,
                (pl.ds(qs + q_block * block_q, block_q),),
                mask=rmask,
                other=PADDING_SEGMENT_ID,
            )
            return rows, rmask, rmask & (seg_q != PADDING_SEGMENT_ID)

        def body_list(i, carry):
            q_block = pl.load(qpidx_ref, (user, lax.div(i, G)))
            _, rmask, rvalid = q_side(q_block)
            allowed = (seg_k > 0)[None, :] & rvalid[:, None]
            return step(i, q_block, rmask, allowed, carry)

        qdlo = lax.div(c0r, block_q)
        qdhi = jnp.minimum(
            lax.div(c0r + block_k + block_q - 1, block_q), lax.div(seqlen_q + block_q - 1, block_q)
        )

        def body_diag(i, carry):
            q_block = qdlo + lax.div(i, G)
            rows, rmask, rvalid = q_side(q_block)
            allowed = (rows[:, None] == cols[None, :]) & (seg_k < 0)[None, :] & rvalid[:, None]
            return step(i, q_block, rmask, allowed, carry)

        carry = (
            jnp.zeros((block_k, k_ref.shape[-1]), jnp.float32),
            jnp.zeros((block_k, k_ref.shape[-1]), jnp.float32),
        )
        carry = lax.fori_loop(0, jnp.where(k_full, G * nqf, 0), body_full, carry)
        carry = lax.fori_loop(0, jnp.where(k_hist & ~k_full, G * nqf, 0), body_full_kmask, carry)
        carry = lax.fori_loop(0, jnp.where(k_hist, G * nqp, 0), body_list, carry)
        dk, dv = lax.fori_loop(0, jnp.where(k_cand, G * (qdhi - qdlo), 0), body_diag, carry)
        pl.store(dk_ref, ksl, (dk * sm_scale).astype(dk_ref.dtype), mask=cmask[:, None])
        pl.store(dv_ref, ksl, dv.astype(dv_ref.dtype), mask=cmask[:, None])


def _v2_backward(
    q,
    k,
    v,
    segment_ids,
    cuq,
    cuk,
    out,
    lse,
    do,
    max_seqlen_q,
    max_seqlen_k,
    sm_scale,
    cap,
    config,
    debug,
):
    total, num_q_heads, head_dim = q.shape
    total_k, num_kv_heads, _ = k.shape
    G = num_q_heads // num_kv_heads

    block_q = _v2_block(config.block_q_dq, max_seqlen_q)
    block_k = _v2_block(config.block_kv_dq, max_seqlen_k)
    meta, num_items = _v2_q_side_meta(
        segment_ids, cuq, cuk, max_seqlen_q, max_seqlen_k, block_q, block_k, total
    )
    args = (meta[0], meta[1], cuq, cuk, segment_ids, *meta[2:], q, k, v, out, do, lse)
    dq, delta = pl.pallas_call(
        functools.partial(
            _v2_dq_kernel,
            sm_scale=sm_scale,
            cap=cap,
            block_q=block_q,
            block_k=block_k,
            q_heads_per_kv_head=G,
            total=total,
        ),
        grid=(num_q_heads, num_items),
        in_specs=[_v2_full_spec(a) for a in args],
        out_specs=[
            pl.BlockSpec((total, num_q_heads, head_dim), lambda *_: (0, 0, 0)),
            pl.BlockSpec((num_q_heads, total), lambda *_: (0, 0)),
        ],
        out_shape=[
            jax.ShapeDtypeStruct(q.shape, q.dtype),
            jax.ShapeDtypeStruct((num_q_heads, total), jnp.float32),
        ],
        compiler_params=CompilerParams(
            num_warps=config.dq_num_warps, num_stages=config.dq_num_stages
        ),
        debug=debug,
        name="mha_backward_q_v2",
    )(*args)

    block_q = _v2_block(config.block_q_dkv, max_seqlen_q)
    block_k = _v2_block(config.block_kv_dkv, max_seqlen_k)
    fq = _v2_block_flags(segment_ids, cuq, max_seqlen_q, block_q)
    qflo, qfhi, qpcnt, qpidx = _v2_range_and_list(fq["all_valid"], fq["has_valid"])
    fk = _v2_block_flags(segment_ids, cuk, max_seqlen_k, block_k)
    kflag = (
        fk["all_hist"].astype(jnp.int32)
        | (fk["has_hist"].astype(jnp.int32) << 1)
        | (fk["has_cand"].astype(jnp.int32) << 2)
    )
    wu, wj, num_items = _v2_work_table(cuk, block_k, max_seqlen_k, total_k)
    args = (wu, wj, cuq, cuk, segment_ids, qflo, qfhi, qpcnt, qpidx, kflag, q, k, v, do, lse, delta)
    dk, dv = pl.pallas_call(
        functools.partial(
            _v2_dkv_kernel,
            sm_scale=sm_scale,
            cap=cap,
            block_q=block_q,
            block_k=block_k,
            q_heads_per_kv_head=G,
            total=total_k,
        ),
        grid=(num_kv_heads, num_items),
        in_specs=[_v2_full_spec(a) for a in args],
        out_specs=[
            pl.BlockSpec((total_k, num_kv_heads, head_dim), lambda *_: (0, 0, 0)),
            pl.BlockSpec((total_k, num_kv_heads, head_dim), lambda *_: (0, 0, 0)),
        ],
        out_shape=[
            jax.ShapeDtypeStruct(k.shape, k.dtype),
            jax.ShapeDtypeStruct(v.shape, v.dtype),
        ],
        compiler_params=CompilerParams(
            num_warps=config.dkv_num_warps, num_stages=config.dkv_num_stages
        ),
        debug=debug,
        name="mha_backward_kv_v2",
    )(*args)
    return dq, dk, dv


@functools.partial(jax.custom_vjp, nondiff_argnums=(7, 8, 9, 10, 11, 12))
def _mha_v2(
    q,
    k,
    v,
    temp,
    segment_ids,
    cu_seqlens_q,
    cu_seqlens_k,
    max_seqlen_q,
    max_seqlen_k,
    sm_scale,
    cap,
    config,
    debug,
):
    out, _ = _v2_forward(
        q,
        k,
        v,
        segment_ids,
        cu_seqlens_q,
        cu_seqlens_k,
        max_seqlen_q,
        max_seqlen_k,
        sm_scale,
        cap,
        config,
        debug,
    )
    return out


def _mha_v2_fwd(
    q,
    k,
    v,
    temp,
    segment_ids,
    cu_seqlens_q,
    cu_seqlens_k,
    max_seqlen_q,
    max_seqlen_k,
    sm_scale,
    cap,
    config,
    debug,
):
    out, lse = _v2_forward(
        q,
        k,
        v,
        segment_ids,
        cu_seqlens_q,
        cu_seqlens_k,
        max_seqlen_q,
        max_seqlen_k,
        sm_scale,
        cap,
        config,
        debug,
    )
    return out, (
        q,
        k,
        v,
        temp,
        segment_ids,
        cu_seqlens_q,
        cu_seqlens_k,
        checkpoint_name(out, PALLAS_ATTN_RESIDUALS_NAME),
        checkpoint_name(lse, PALLAS_ATTN_RESIDUALS_NAME),
    )


def _mha_v2_bwd(max_seqlen_q, max_seqlen_k, sm_scale, cap, config, debug, res, do):
    q, k, v, temp, segment_ids, cuq, cuk, out, lse = res
    dq, dk, dv = _v2_backward(
        q,
        k,
        v,
        segment_ids,
        cuq,
        cuk,
        out,
        lse,
        do,
        max_seqlen_q,
        max_seqlen_k,
        sm_scale,
        cap,
        config,
        debug,
    )
    dtemp = jnp.zeros(temp.shape, temp.dtype)
    dsegment = jnp.zeros(segment_ids.shape, q.dtype)
    return dq, dk, dv, dtemp, dsegment, None, None


_mha_v2.defvjp(_mha_v2_fwd, _mha_v2_bwd)


@functools.partial(
    jax.jit,
    static_argnames=[
        "max_seqlen_q",
        "max_seqlen_k",
        "sm_scale",
        "cap",
        "cap_method",
        "causal",
        "window_len",
        "z_loss_weight",
        "config",
        "backward_pass_impl",
        "grid",
        "interpret",
        "debug",
    ],
)
def mha(
    q,
    k,
    v,
    temp,
    segment_ids,
    cu_seqlens_q,
    cu_seqlens_k,
    max_seqlen_q: int,
    max_seqlen_k: int,
    sm_scale: float = 1.0,
    cap: float = -1.0,
    cap_method: str = "tanh",
    causal: bool = True,
    window_len: int = -1,
    z_loss_weight: float = 0.0,
    config: TuningConfig | V2TuningConfig | None = None,
    backward_pass_impl: str = "triton_split",
    grid=None,
    interpret: bool = False,
    debug: bool = False,
):
    assert q.shape[0] == k.shape[0] == segment_ids.shape[0], "packed lengths must match"
    assert q.shape[1] % k.shape[1] == 0, "num_q_heads must be divisible by num_kv_heads"
    assert v.shape == k.shape and k.shape[2] == q.shape[2], "k/v shapes must match"
    assert temp.shape == (q.shape[0],), "temp length must match q"
    if window_len >= 0:
        raise NotImplementedError("Packed ranker attention does not support window_len >= 0")
    if max_seqlen_q == 0 and max_seqlen_k == 0:
        return jnp.zeros_like(q)
    if config is None:
        config = _default_config()
    if isinstance(config, V2TuningConfig):
        if (
            v.dtype == q.dtype
            and window_len == -1
            and max_seqlen_q == max_seqlen_k
            and backward_pass_impl == "triton_split"
            and _v2_supported(q, k, cap, cap_method, causal, z_loss_weight, grid, interpret)
        ):
            if cu_seqlens_q.shape == cu_seqlens_k.shape:
                return lax.cond(
                    jnp.all(cu_seqlens_q == cu_seqlens_k),
                    lambda: _mha_v2(
                        q,
                        k,
                        v,
                        temp,
                        segment_ids,
                        cu_seqlens_q,
                        cu_seqlens_k,
                        max_seqlen_q,
                        max_seqlen_k,
                        float(sm_scale),
                        float(cap),
                        config,
                        debug,
                    ),
                    lambda: _mha_v1(
                        q,
                        k,
                        v,
                        temp,
                        segment_ids,
                        cu_seqlens_q,
                        cu_seqlens_k,
                        max_seqlen_q,
                        max_seqlen_k,
                        sm_scale,
                        cap,
                        cap_method,
                        causal,
                        window_len,
                        z_loss_weight,
                        _v1_config(),
                        backward_pass_impl,
                        grid,
                        interpret,
                        debug,
                    ),
                )
        config = _v1_config()
    return _mha_v1(
        q,
        k,
        v,
        temp,
        segment_ids,
        cu_seqlens_q,
        cu_seqlens_k,
        max_seqlen_q,
        max_seqlen_k,
        sm_scale,
        cap,
        cap_method,
        causal,
        window_len,
        z_loss_weight,
        config,
        backward_pass_impl,
        grid,
        interpret,
        debug,
    )
