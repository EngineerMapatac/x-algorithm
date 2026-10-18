# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from jax._src.deprecations import deprecation_getattr


def patch_xla_extensions():
    from jax import errors
    from jax.lib import xla_extension

    if hasattr(xla_extension, "XlaRuntimeError"):
        return

    deprecations = xla_extension._deprecations
    if "XlaRuntimeError" in deprecations:
        deprecations.pop("XlaRuntimeError")
    xla_extension.__getattr__ = deprecation_getattr(xla_extension.__name__, deprecations)
    xla_extension.XlaRuntimeError = errors.JaxRuntimeError


patch_xla_extensions()
