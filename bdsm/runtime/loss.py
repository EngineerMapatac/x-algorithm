from __future__ import annotations

import jax
import jax.numpy as jnp


def masked_sce_focal_loss(
    logits: jax.Array,
    label_vector: jax.Array,
    label_mask: jax.Array,
    label_smoothing: float,
    reverse_ce_weight: float,
    focal_gamma: float,
) -> tuple[jax.Array, dict[str, jax.Array]]:
    logits = logits.astype(jnp.float32)
    targets = label_vector.astype(jnp.float32)
    mask = label_mask.astype(jnp.float32)

    smooth_targets = (1.0 - label_smoothing) * targets + label_smoothing * 0.5

    probs = jax.nn.sigmoid(logits)
    probs_clipped = jnp.clip(probs, 1e-7, 1.0 - 1e-7)
    ce = -(
        smooth_targets * jnp.log(probs_clipped)
        + (1.0 - smooth_targets) * jnp.log(1.0 - probs_clipped)
    )

    p_t = targets * probs_clipped + (1.0 - targets) * (1.0 - probs_clipped)
    focal_weight = (1.0 - p_t) ** focal_gamma if focal_gamma > 0 else jnp.ones_like(ce)
    ce = focal_weight * ce

    smooth_clipped = jnp.clip(smooth_targets, 1e-7, 1.0 - 1e-7)
    rce = -(probs * jnp.log(smooth_clipped) + (1.0 - probs) * jnp.log(1.0 - smooth_clipped))
    rce = focal_weight * rce

    sce = (ce + reverse_ce_weight * rce) * mask
    num_evaluated = jnp.sum(mask) + 1e-10
    loss = jnp.sum(sce) / num_evaluated

    per_head_mask_sum = jnp.sum(mask, axis=0) + 1e-10
    stats = {
        "bce_loss": loss,
        "num_evaluated_pairs": jnp.sum(mask),
        "per_head_loss": jnp.sum(sce, axis=0) / per_head_mask_sum,
        "per_head_pos_frac": jnp.sum(targets * mask, axis=0) / per_head_mask_sum,
        "per_head_mean_pred": jnp.sum(probs * mask, axis=0) / per_head_mask_sum,
    }
    if focal_gamma > 0:
        stats["focal_mean_weight"] = jnp.sum(focal_weight * mask) / num_evaluated
    return loss, stats
