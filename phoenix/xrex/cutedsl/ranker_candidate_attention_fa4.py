# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import jax
import jax.numpy as jnp
from jax.ad_checkpoint import checkpoint_name

_CANDIDATE_ATTENTION_CACHE: dict = {}


def _candidate_attention_fwd_bwd(q_shape, k_shape, num_seqs, sm_scale):
    from xrex.cutedsl.ranker_attention_fa4 import cutedsl_arch
    from xrex.cutedsl.ranker_attention_varlen_fa4 import zeros_after

    total_q, num_q_heads, head_dim = q_shape
    total_k, num_kv_heads, _ = k_shape
    arch = cutedsl_arch()
    sm_scale = float(sm_scale)
    cache_key = (arch, total_q, total_k, num_seqs, num_q_heads, num_kv_heads, head_dim, sm_scale)
    if cache_key in _CANDIDATE_ATTENTION_CACHE:
        return _CANDIDATE_ATTENTION_CACHE[cache_key]

    import cuda.bindings.driver as cuda_driver
    import cutlass
    import cutlass.cute as cute
    from cutlass.jax import cutlass_call

    from xrex.cutedsl.ranker_fa4.flash_bwd_postprocess import FlashAttentionBackwardPostprocess
    from xrex.cutedsl.ranker_fa4.flash_bwd_preprocess import FlashAttentionBackwardPreprocess

    if arch != 90:
        raise NotImplementedError("ranker candidate attention: only SM90 (H100) is supported")
    from xrex.cutedsl.ranker_fa4.flash_bwd_sm90 import FlashAttentionBackwardSm90
    from xrex.cutedsl.ranker_fa4.flash_fwd_sm90 import FlashAttentionForwardSm90

    qpk = num_q_heads // num_kv_heads
    tile_n = 128
    bwd_tile_m = 64
    hdr = ((head_dim + 31) // 32) * 32
    total_q_padded = (total_q + num_seqs * bwd_tile_m + bwd_tile_m - 1) // bwd_tile_m * bwd_tile_m
    total_k_padded = (total_k + num_seqs * tile_n + tile_n - 1) // tile_n * tile_n

    fa_fwd = FlashAttentionForwardSm90(
        cutlass.BFloat16,
        head_dim,
        head_dim,
        qpk,
        is_causal=False,
        is_local=False,
        pack_gqa=qpk > 1,
        tile_m=128,
        tile_n=tile_n,
        num_stages=2,
        num_threads=384,
        Q_in_regs=False,
        intra_wg_overlap=True,
        mma_pv_is_rs=True,
    )

    @cute.jit
    def launch_fwd(
        stream: cuda_driver.CUstream,
        mQ,
        mK,
        mV,
        mCuQ,
        mCuK,
        mSeqUsedK,
        mO,
        mLSE,
        softmax_scale: cutlass.Float32,
    ):
        fa_fwd(mQ, mK, mV, mO, mLSE, softmax_scale, mCuQ, mCuK, None, mSeqUsedK, stream=stream)

    fwd_call = cutlass_call(
        launch_fwd,
        output_shape_dtype=[
            jax.ShapeDtypeStruct((total_q, num_q_heads, head_dim), jnp.bfloat16),
            jax.ShapeDtypeStruct((num_q_heads, total_q), jnp.float32),
        ],
        use_static_tensors=False,
        softmax_scale=cutlass.Float32(sm_scale),
    )

    fa_pre = FlashAttentionBackwardPreprocess(
        cutlass.BFloat16, head_dim, head_dim, tile_m=bwd_tile_m
    )

    @cute.jit
    def launch_pre(stream: cuda_driver.CUstream, mO, mdO, mLSE, mCuQ, mdPsum, mLSElog2):
        fa_pre(mO, mdO, mdPsum, mLSE, mLSElog2, None, mCuQ, None, None, stream)

    pre_call = cutlass_call(
        launch_pre,
        output_shape_dtype=[
            jax.ShapeDtypeStruct((num_q_heads, total_q_padded), jnp.float32),
            jax.ShapeDtypeStruct((num_q_heads, total_q_padded), jnp.float32),
        ],
        use_static_tensors=False,
    )

    bwd_atom_layout_dkv = 2
    fa_bwd = FlashAttentionBackwardSm90(
        cutlass.BFloat16,
        head_dim,
        head_dim,
        qpk,
        False,
        is_local=False,
        deterministic=False,
        tile_m=bwd_tile_m,
        tile_n=tile_n,
        Q_stage=2,
        dO_stage=2,
        PdS_stage=2,
        SdP_swapAB=True,
        dKV_swapAB=False,
        dQ_swapAB=False,
        AtomLayoutMSdP=1,
        AtomLayoutNdKV=bwd_atom_layout_dkv,
        AtomLayoutMdQ=1,
        num_threads=384,
    )
    dkv_postprocess = qpk > 1
    dkv_out = jax.ShapeDtypeStruct(
        (num_kv_heads, total_k_padded * hdr)
        if dkv_postprocess
        else (total_k, num_kv_heads, head_dim),
        jnp.float32 if dkv_postprocess else jnp.bfloat16,
    )
    dq_accum_shape = (num_q_heads, total_q_padded * hdr)

    @cute.jit
    def launch_bwd(
        stream: cuda_driver.CUstream,
        mQ,
        mK,
        mV,
        mdO,
        mLSElog2,
        mdPsum,
        mCuQ,
        mCuK,
        mSeqUsedK,
        mdQaccum,
        mdK,
        mdV,
        softmax_scale: cutlass.Float32,
    ):
        fa_bwd(
            mQ,
            mK,
            mV,
            mdO,
            mLSElog2,
            mdPsum,
            mdQaccum,
            mdK,
            mdV,
            softmax_scale,
            mCuQ,
            mCuK,
            None,
            mSeqUsedK,
            stream=stream,
        )

    bwd_call = cutlass_call(
        launch_bwd,
        output_shape_dtype=[
            jax.ShapeDtypeStruct(dq_accum_shape, jnp.float32),
            dkv_out,
            dkv_out,
        ],
        input_output_aliases={9: 0, 10: 1, 11: 2},
        use_static_tensors=False,
        softmax_scale=cutlass.Float32(sm_scale),
    )

    def postprocess(tile_m, atom_layout, out_shape, scale, with_seqused):
        fa_post = FlashAttentionBackwardPostprocess(
            cutlass.BFloat16,
            head_dim,
            arch,
            tile_m=tile_m,
            num_threads=256,
            AtomLayoutMdQ=atom_layout,
        )

        if with_seqused:

            @cute.jit
            def launch(
                stream: cuda_driver.CUstream, mAccum, mCu, mSeqUsed, mOut, scale: cutlass.Float32
            ):
                fa_post(mAccum, mOut, scale, mCu, mSeqUsed, stream)

        else:

            @cute.jit
            def launch(stream: cuda_driver.CUstream, mAccum, mCu, mOut, scale: cutlass.Float32):
                fa_post(mAccum, mOut, scale, mCu, None, stream)

        return cutlass_call(
            launch,
            output_shape_dtype=[jax.ShapeDtypeStruct(out_shape, jnp.bfloat16)],
            use_static_tensors=False,
            scale=cutlass.Float32(scale),
        )

    kv_shape = (total_k, num_kv_heads, head_dim)
    post_dq = postprocess(bwd_tile_m, 1, (total_q, num_q_heads, head_dim), sm_scale, False)
    post_dk = post_dv = None
    if dkv_postprocess:
        post_dk = postprocess(tile_n, bwd_atom_layout_dkv, kv_shape, sm_scale, True)
        post_dv = postprocess(tile_n, bwd_atom_layout_dkv, kv_shape, 1.0, True)

    def self_scores(q, k_self, query_valid):
        k_rep = jnp.repeat(k_self, qpk, axis=1)
        s = jnp.einsum("thd,thd->th", q.astype(jnp.float32), k_rep.astype(jnp.float32))
        return jnp.where(query_valid[:, None], s * sm_scale, -jnp.inf)

    def forward(q, k_keys, v_keys, k_self, v_self, cu_q, cu_k, used_k):
        query_valid = jnp.arange(total_q) < cu_q[-1]
        o_hist, lse_hist = fwd_call(q, k_keys, v_keys, cu_q, cu_k, used_k)
        lse_hist = jnp.where(query_valid[:, None], lse_hist.T, -jnp.inf)
        s = self_scores(q, k_self, query_valid)
        lse = jnp.logaddexp(lse_hist, s)
        lse_safe = jnp.where(query_valid[:, None], lse, 0.0)
        w_hist = jnp.where(query_valid[:, None], jnp.exp(lse_hist - lse_safe), 0.0)
        w_self = jnp.where(query_valid[:, None], jnp.exp(s - lse_safe), 0.0)
        v_rep = jnp.repeat(v_self, qpk, axis=1).astype(jnp.float32)
        o_hist = jnp.where(query_valid[:, None, None], o_hist.astype(jnp.float32), 0.0)
        o = (w_hist[..., None] * o_hist + w_self[..., None] * v_rep).astype(jnp.bfloat16)
        return o, lse_safe

    def backward(q, k_keys, v_keys, k_self, v_self, cu_q, cu_k, used_k, o, lse, do):
        query_valid = jnp.arange(total_q) < cu_q[-1]
        do = jnp.where(query_valid[:, None, None], do, 0).astype(jnp.bfloat16)
        dpsum, lse_log2 = pre_call(o, do, lse.T, cu_q)
        dq_accum = zeros_after(dq_accum_shape, jnp.float32, do)
        dk_init = zeros_after(dkv_out.shape, dkv_out.dtype, do)
        dv_init = zeros_after(dkv_out.shape, dkv_out.dtype, do)
        dq_accum, dk_keys, dv_keys = bwd_call(
            q,
            k_keys,
            v_keys,
            do,
            lse_log2,
            dpsum,
            cu_q,
            cu_k,
            used_k,
            dq_accum,
            dk_init,
            dv_init,
        )
        (dq,) = post_dq(dq_accum, cu_q)
        if post_dk is not None and post_dv is not None:
            (dk_keys,) = post_dk(dk_keys, cu_k, used_k)
            (dv_keys,) = post_dv(dv_keys, cu_k, used_k)
        marks = jnp.zeros(total_k + 1, jnp.int32).at[cu_k[:-1]].add(1)
        marks = marks.at[cu_k[:-1] + used_k].add(-1)
        key_valid = jnp.cumsum(marks[:-1]) > 0
        dq = jnp.where(query_valid[:, None, None], dq, 0)
        dk_keys = jnp.where(key_valid[:, None, None], dk_keys, 0)
        dv_keys = jnp.where(key_valid[:, None, None], dv_keys, 0)

        s = self_scores(q, k_self, query_valid)
        p_self = jnp.where(query_valid[:, None], jnp.exp(s - lse), 0.0)
        do32 = do.astype(jnp.float32)
        delta = jnp.sum(do32 * o.astype(jnp.float32), axis=-1)
        v_rep = jnp.repeat(v_self, qpk, axis=1).astype(jnp.float32)
        ds = p_self * (jnp.sum(do32 * v_rep, axis=-1) - delta)
        ds = jnp.where(query_valid[:, None], ds, 0.0)
        k_rep = jnp.repeat(k_self, qpk, axis=1).astype(jnp.float32)
        dq = dq.astype(jnp.float32) + ds[..., None] * k_rep * sm_scale

        def group_sum(x):
            return x.reshape(total_q, num_kv_heads, qpk, head_dim).sum(axis=2)

        dk_self = group_sum(ds[..., None] * q.astype(jnp.float32) * sm_scale)
        dv_self = group_sum(p_self[..., None] * do32)
        return (
            dq.astype(q.dtype),
            dk_keys.astype(k_keys.dtype),
            dv_keys.astype(v_keys.dtype),
            dk_self.astype(k_self.dtype),
            dv_self.astype(v_self.dtype),
        )

    _CANDIDATE_ATTENTION_CACHE[cache_key] = (forward, backward)
    return forward, backward


def ranker_candidate_attention_fa4(
    q,
    k,
    v,
    sm_scale,
    block_sparse_layout,
    valid_block_upper,
    valid_block_lower,
    candidate_cu_seqlens,
    candidate_key_starts,
    candidate_key_counts,
    history_region_len,
):
    from xrex.cutedsl.ranker_attention_varlen_fa4 import (
        varlen_backward,
        varlen_forward,
        varlen_kernels,
    )

    assert q.shape[0] == 1, "one packed sequence per device"
    region_start = int(history_region_len)
    kernels, bs_args = varlen_kernels(
        q, k, v, sm_scale, block_sparse_layout, valid_block_upper, valid_block_lower
    )
    cand_forward, cand_backward = _candidate_attention_fwd_bwd(
        q[0, region_start:].shape, k[0].shape, candidate_cu_seqlens.shape[0] - 1, sm_scale
    )

    def candidate_inputs(q, k, v, cand_cu, key_starts, key_counts):
        keys, values = k[0], v[0]
        return (
            q[0, region_start:],
            keys,
            values,
            keys[region_start:],
            values[region_start:],
            cand_cu,
            key_starts,
            key_counts,
        )

    def forward(q, k, v, cand_cu, key_starts, key_counts, bs_args):
        out, lse = varlen_forward(kernels, q, k, v, bs_args)
        cand_o, cand_lse = cand_forward(*candidate_inputs(q, k, v, cand_cu, key_starts, key_counts))
        return out.at[0, region_start:].set(cand_o), lse, cand_lse

    @jax.custom_vjp
    def attention(q, k, v, cand_cu, key_starts, key_counts, *bs_args):
        return forward(q, k, v, cand_cu, key_starts, key_counts, bs_args)[0]

    def attention_fwd(q, k, v, cand_cu, key_starts, key_counts, *bs_args):
        out, lse, cand_lse = forward(q, k, v, cand_cu, key_starts, key_counts, bs_args)
        out = checkpoint_name(out, "cutedsl_attn_outputs")
        lse = checkpoint_name(lse, "cutedsl_attn_outputs")
        cand_lse = checkpoint_name(cand_lse, "cutedsl_attn_outputs")
        return out, (q, k, v, cand_cu, key_starts, key_counts, out, lse, cand_lse, bs_args)

    def attention_bwd(res, g):
        q, k, v, cand_cu, key_starts, key_counts, out, lse, cand_lse, bs_args = res
        dq, dk, dv = varlen_backward(kernels, q, k, v, out, lse, bs_args, g)
        dq_cand, dk_keys, dv_keys, dk_self, dv_self = cand_backward(
            *candidate_inputs(q, k, v, cand_cu, key_starts, key_counts),
            out[0, region_start:],
            cand_lse,
            g[0, region_start:],
        )
        dq = dq.at[0, region_start:].set(dq_cand)
        dk = (dk.at[0].add(dk_keys)).at[0, region_start:].add(dk_self)
        dv = (dv.at[0].add(dv_keys)).at[0, region_start:].add(dv_self)
        return (dq, dk, dv, None, None, None) + (None,) * len(bs_args)

    attention.defvjp(attention_fwd, attention_bwd)
    return attention(
        q, k, v, candidate_cu_seqlens, candidate_key_starts, candidate_key_counts, *bs_args
    )
