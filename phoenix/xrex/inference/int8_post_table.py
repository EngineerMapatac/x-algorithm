# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import jax
import jax.numpy as jnp


@jax.jit
def quantize_post_table(t: jax.Array) -> tuple[jax.Array, jax.Array]:
    s = jnp.maximum(jnp.max(jnp.abs(t), axis=1).astype(jnp.float32) / 127.0, 1e-12)
    q = jnp.clip(jnp.round(t.astype(jnp.float32) / s[:, None]), -127, 127).astype(jnp.int8)
    return q, s
