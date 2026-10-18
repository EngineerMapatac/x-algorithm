# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import zlib

import jax
import numpy as np
from jax import numpy as jnp
from jax.sharding import PartitionSpec as P


@jax.jit
def adler32(data: jax.Array) -> jax.Array:
    def _host(arr):
        return np.uint32(zlib.adler32(np.asarray(arr).tobytes()))

    return jax.pure_callback(_host, jax.ShapeDtypeStruct((), jnp.uint32), data)


def global_adler32(data: jax.Array, mesh, pspec) -> jax.Array:
    if not data.shape:
        return adler32(data)
    gathered = jax.lax.with_sharding_constraint(data, jax.sharding.NamedSharding(mesh, P()))
    return adler32(gathered)


def global_adler32_fast(data: jax.Array, mesh, pspec) -> jax.Array:
    return global_adler32(data, mesh, pspec)
