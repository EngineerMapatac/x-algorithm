# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import logging
import zlib

import jax
import numpy as np
from jax import numpy as jnp

_log = logging.getLogger(__name__)

_BASE = 65521

_FFI_BACKENDS: set[str] = set()

try:
    import xrex_cuda_kernels.adler32_api as adler32_api
except ModuleNotFoundError:
    adler32_api = None
else:
    jax.ffi.register_ffi_target("xrex_adler32", adler32_api.adler32(), platform="CUDA")
    _FFI_BACKENDS.add("gpu")

try:
    import xrex_cuda_kernels.adler32_cpu_api as adler32_cpu_api
except ModuleNotFoundError:
    adler32_cpu_api = None
else:
    jax.ffi.register_ffi_target("xrex_adler32", adler32_cpu_api.adler32(), platform="cpu")
    _FFI_BACKENDS.add("cpu")

_MISSING = [
    name
    for name, api in (("adler32_api", adler32_api), ("adler32_cpu_api", adler32_cpu_api))
    if api is None
]
if _MISSING:
    _log.warning(
        "xrex.cuda.adler32: xrex_cuda_kernels.%s not installed; the zlib pure_callback "
        "reference path runs instead of the compiled kernel(s)",
        ", ".join(_MISSING),
    )


def _adler32_reference(data: jax.Array) -> jax.Array:
    def _host(arr):
        return np.uint32(zlib.adler32(np.asarray(arr).tobytes()))

    return jax.pure_callback(_host, jax.ShapeDtypeStruct((), jnp.uint32), data)


def _adler32_ffi(data: jax.Array) -> jax.Array:
    call = jax.ffi.ffi_call(
        "xrex_adler32",
        (jax.ShapeDtypeStruct((), np.uint32), jax.ShapeDtypeStruct((), np.uint32)),
        vmap_method="broadcast_all",
    )
    a, b = call(data)
    a %= _BASE
    b %= _BASE
    return (b << 16) | a


@jax.jit
def adler32(data: jax.Array) -> jax.Array:
    if jax.default_backend() in _FFI_BACKENDS:
        return _adler32_ffi(data)
    return _adler32_reference(data)
