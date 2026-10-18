# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import chex
import jax


@chex.dataclass(kw_only=True)
class MemoryModelOutput:
    output: jax.Array
    query_heads: jax.Array | None = None
    key_heads: jax.Array | None = None
    value_heads: jax.Array | None = None
    attn_output: jax.Array | None = None
