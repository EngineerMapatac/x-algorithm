# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from typing import cast

import jax
import jax.numpy as jnp
from jax.sharding import PartitionSpec as P

from xrex.cuda.unique import unique


def compress_token_ids(
    token_ids: jax.Array,
    fill_size: int,
    fill_value: int,
    output_sharding: P = P(),
) -> tuple[jax.Array, jax.Array]:
    all_tokens_ids = token_ids.reshape((-1,))
    all_tokens_ids = jnp.concatenate([all_tokens_ids, jnp.array([0])])
    all_tokens_ids = jax.lax.with_sharding_constraint(all_tokens_ids, P())

    unique_tokens, inverse_indices = unique(
        all_tokens_ids,
        return_inverse=True,
        size=fill_size,
        fill_value=fill_value,
    )
    unique_tokens = cast(jax.Array, unique_tokens)
    inverse_indices = cast(jax.Array, inverse_indices)

    inverse_indices = inverse_indices[:-1]

    inverse_indices = jax.lax.with_sharding_constraint(inverse_indices, output_sharding)
    inverse_indices = cast(jax.Array, jnp.where(inverse_indices == 0, -1, inverse_indices))

    return unique_tokens, inverse_indices
