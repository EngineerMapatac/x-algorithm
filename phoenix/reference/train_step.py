# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from __future__ import annotations

from typing import Any

import jax
import jax.numpy as jnp

from xrex.models.compress_token_ids import compress_token_ids
from xrex.models.model_utils import Parameter
from xrex.models.recsys_embedding import EmbTable, RecsysEmbeddingsParameter
from xrex.optimizers.optim import apply_updates


def _hash_leaves(batch: Any, use_ip: bool) -> list[jax.Array]:
    leaves = [
        batch["user_hashes"],
        batch["history_seq"]["post_hashes"],
        batch["history_seq"]["auth_hashes"],
        batch["candidate_seq"]["post_hashes"],
        batch["candidate_seq"]["auth_hashes"],
    ]
    if use_ip:
        leaves.append(batch["user_ip_hashes"])
    return leaves


def flatten_token_ids(batch: Any, use_ip: bool) -> jax.Array:
    batch_size = batch["user_hashes"].shape[0]
    return jnp.concatenate([x.reshape(batch_size, -1) for x in _hash_leaves(batch, use_ip)], axis=1)


def lookup_embeddings(
    emb_table: Parameter, batch: Any, use_ip: bool
) -> tuple[RecsysEmbeddingsParameter, jax.Array]:
    leaves = _hash_leaves(batch, use_ip)
    batch_size = leaves[0].shape[0]
    flat = [x.reshape(batch_size, -1) for x in leaves]
    segment_lengths = [x.shape[1] for x in flat]
    all_hashes = jnp.concatenate(flat, axis=1)
    all_x = emb_table.x[all_hashes]
    splits = jnp.split(all_x, list(_cumsum(segment_lengths[:-1])), axis=1)
    reshaped = [s.reshape(*leaf.shape, all_x.shape[-1]) for s, leaf in zip(splits, leaves)]
    user_x, hist_post_x, hist_auth_x, cand_post_x, cand_auth_x, *rest = reshaped
    embeddings = RecsysEmbeddingsParameter(
        user_embeddings=EmbTable(x=user_x),
        history_post_embeddings=EmbTable(x=hist_post_x),
        history_author_embeddings=EmbTable(x=hist_auth_x),
        candidate_post_embeddings=EmbTable(x=cand_post_x),
        candidate_author_embeddings=EmbTable(x=cand_auth_x),
        user_ip_embeddings=EmbTable(x=rest[0]) if use_ip else None,
    )
    return embeddings, all_hashes


def _cumsum(xs: list[int]) -> list[int]:
    out, acc = [], 0
    for x in xs:
        acc += x
        out.append(acc)
    return out


def segment_sum_embeddings(
    emb_grads: RecsysEmbeddingsParameter, inverse_indices: jax.Array, num_unique: int, use_ip: bool
) -> jax.Array:
    leaves = [
        emb_grads.user_embeddings.x,
        emb_grads.history_post_embeddings.x,
        emb_grads.history_author_embeddings.x,
        emb_grads.candidate_post_embeddings.x,
        emb_grads.candidate_author_embeddings.x,
    ]
    if use_ip and emb_grads.user_ip_embeddings is not None:
        leaves.append(emb_grads.user_ip_embeddings.x)
    dim = leaves[0].shape[-1]
    batch_size = leaves[0].shape[0]
    grads_flat = jnp.concatenate([x.reshape(batch_size, -1, dim) for x in leaves], axis=1).reshape(
        -1, dim
    )
    return jax.ops.segment_sum(
        grads_flat.astype(jnp.float32), inverse_indices, num_segments=num_unique
    ).astype(jnp.bfloat16)


def compute_grad_norm(grads: Any) -> jax.Array:
    norms_sq = jax.tree.map(lambda x: jnp.sum(jnp.square(x)), grads)
    leaves = jax.tree_util.tree_leaves(norms_sq)
    return jnp.sqrt(jnp.sum(jnp.stack(leaves)))


def is_valid_step(grads: Any, keep_threshold: float | None = None) -> tuple[jax.Array, jax.Array]:
    grad_norm = compute_grad_norm(grads)
    valid = jnp.all(jnp.isfinite(grad_norm))
    if keep_threshold:
        valid = jnp.logical_and(valid, jnp.all(jnp.less(grad_norm, keep_threshold)))
    return valid, grad_norm


def recsys_train_step(
    loss_apply: Any,
    dense_optim: Any,
    emb_optim: Any,
    params: Any,
    opt_state: Any,
    emb_table: Parameter,
    emb_table_state: Any,
    rng: jax.Array,
    batch: Any,
    lr: float,
    *,
    use_ip: bool,
    hash_vocab: int,
    keep_threshold: float | None = None,
) -> tuple[Any, Any, Parameter, Any, jax.Array]:
    embeddings, all_hashes = lookup_embeddings(emb_table, batch, use_ip)

    num_unique = min(all_hashes.size, hash_vocab) + 1
    unique_tokens, inverse_indices = compress_token_ids(
        all_hashes, fill_size=num_unique, fill_value=hash_vocab
    )

    (loss, _stats), (dense_grads, emb_grads) = jax.value_and_grad(
        loss_apply, argnums=(0, 3), has_aux=True
    )(params, rng, batch, embeddings)

    valid_step, _grad_norm = is_valid_step(dense_grads, keep_threshold)

    updates, new_opt_state = dense_optim.update(dense_grads, opt_state, params=params)
    new_params = apply_updates(params, updates, lr)
    new_params = jax.tree.map(lambda u, o: jnp.where(valid_step, u, o), new_params, params)
    new_opt_state = jax.tree.map(lambda u, o: jnp.where(valid_step, u, o), new_opt_state, opt_state)

    per_unique_grad = Parameter(
        segment_sum_embeddings(emb_grads, inverse_indices, num_unique, use_ip),
        **emb_table.fields,
    )
    emb_valid_step, _ = is_valid_step(per_unique_grad)
    new_emb_table, new_emb_state, _metrics = emb_optim.sparse_update(
        grads=per_unique_grad,
        full_state=emb_table_state,
        full_emb_table=emb_table,
        unique_tokens=unique_tokens,
        num_unique=num_unique,
        lr=lr,
        valid_step=emb_valid_step,
    )
    return new_params, new_opt_state, new_emb_table, new_emb_state, loss
