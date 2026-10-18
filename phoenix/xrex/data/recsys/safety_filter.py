# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from __future__ import annotations

from typing import Literal

import jax
import jax.numpy as jnp
import jax.typing

SafetyFilterMode = Literal["off", "hard", "soft"]


def apply_safety_filter(
    safety_label_mask: jax.typing.ArrayLike | None,
    padding_mask: jax.Array,
    raw_weights: jax.Array | None,
    *,
    mode: SafetyFilterMode,
    bits: int,
    soft_weight: float,
) -> tuple[jax.Array, jax.Array | None]:
    if mode == "off" or safety_label_mask is None:
        return padding_mask, raw_weights

    flagged = (safety_label_mask & bits) != 0

    if mode == "hard":
        return padding_mask & ~flagged, raw_weights

    if mode == "soft":
        weight_factor = jnp.where(flagged, soft_weight, 1.0).astype(jnp.float32)
        if raw_weights is None:
            return padding_mask, weight_factor
        return padding_mask, raw_weights.astype(jnp.float32) * weight_factor

    raise ValueError(f"Unknown safety_filter_mode: {mode!r}")


def safety_filter_stats(
    safety_label_mask: jax.typing.ArrayLike | None,
    padding_mask: jax.Array,
    *,
    bits: int,
    prefix: str,
) -> dict[str, jax.Array]:
    if safety_label_mask is None:
        zero = jnp.float32(0.0)
        return {
            f"{prefix}_frac_flagged": zero,
            f"{prefix}_frac_flagged_of_valid": zero,
        }
    flagged = ((safety_label_mask & bits) != 0).astype(jnp.float32)
    valid = padding_mask.astype(jnp.float32)
    frac_flagged = jnp.mean(flagged)
    valid_sum = jnp.sum(valid)
    frac_flagged_of_valid = jnp.sum(flagged * valid) / jnp.maximum(valid_sum, 1.0)
    return {
        f"{prefix}_frac_flagged": frac_flagged,
        f"{prefix}_frac_flagged_of_valid": frac_flagged_of_valid,
    }
