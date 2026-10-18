# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import logging

import jax
import jax.numpy as jnp

try:
    import xrex_cuda_kernels.unique_api as unique_api
except ModuleNotFoundError:
    unique_api = None
    logging.getLogger(__name__).warning(
        "xrex.cuda.unique: xrex_cuda_kernels.unique_api not installed; the jnp.unique "
        "reference path runs instead of the compiled kernel"
    )
else:
    jax.ffi.register_ffi_target("xrex_unique", fn=unique_api.unique(), platform="CUDA")


def unique(
    x: jax.Array, return_inverse: bool, size: int, fill_value: int
) -> tuple[jax.Array, jax.Array]:
    if x.dtype != jnp.int32:
        raise ValueError("Only int32 is supported for unique.")
    if x.ndim != 1:
        raise ValueError("Only 1D arrays are supported for unique.")

    if unique_api is None or jax.default_backend() != "gpu":
        if return_inverse:
            unique_vals, unique_inverse = jnp.unique(
                x, return_inverse=True, size=size, fill_value=fill_value
            )
        else:
            unique_vals = jnp.unique(x, size=size, fill_value=fill_value)
            unique_inverse = jnp.zeros_like(x)
        return unique_vals.astype(x.dtype), unique_inverse.astype(x.dtype)

    call = jax.ffi.ffi_call(
        "xrex_unique",
        [
            jax.ShapeDtypeStruct(shape=[size], dtype=x.dtype),
            jax.ShapeDtypeStruct(shape=x.shape, dtype=x.dtype),
            jax.ShapeDtypeStruct(shape=x.shape, dtype=x.dtype),
            jax.ShapeDtypeStruct(shape=x.shape, dtype=x.dtype),
            jax.ShapeDtypeStruct(shape=x.shape, dtype=x.dtype),
        ],
    )
    unique_vals, unique_inverse, _, _, _ = call(
        x, return_inverse=return_inverse, size=size, fill_value=fill_value
    )
    return unique_vals, unique_inverse
