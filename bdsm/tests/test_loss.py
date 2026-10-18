import numpy as np
import pytest

jax = pytest.importorskip("jax")
jnp = jax.numpy

from loss import masked_sce_focal_loss


def _toy():
    logits = jnp.asarray([[2.0, -2.0], [0.0, 1.0]])
    targets = jnp.asarray([[1.0, 0.0], [0.0, 1.0]])
    mask = jnp.ones_like(targets)
    return logits, targets, mask


def test_focal_stats_present_iff_gamma_positive():
    logits, targets, mask = _toy()
    _, stats_on = masked_sce_focal_loss(logits, targets, mask, 0.1, 0.5, focal_gamma=2.0)
    assert "focal_mean_weight" in stats_on
    assert 0.0 < float(stats_on["focal_mean_weight"]) < 1.0
    _, stats_off = masked_sce_focal_loss(logits, targets, mask, 0.1, 0.5, focal_gamma=0.0)
    assert "focal_mean_weight" not in stats_off


def test_focal_downweights_easy_examples():
    logits, targets, mask = _toy()
    loss_plain, _ = masked_sce_focal_loss(logits, targets, mask, 0.0, 0.0, focal_gamma=0.0)
    loss_focal, _ = masked_sce_focal_loss(logits, targets, mask, 0.0, 0.0, focal_gamma=2.0)
    assert float(loss_focal) < float(loss_plain)


def test_masked_pairs_contribute_nothing():
    logits, targets, mask = _toy()
    base, _ = masked_sce_focal_loss(logits, targets, mask, 0.1, 0.5, 2.0)
    logits2 = logits.at[0, 1].set(50.0)
    mask2 = mask.at[0, 1].set(0.0)
    ref, _ = masked_sce_focal_loss(logits, targets, mask2, 0.1, 0.5, 2.0)
    poisoned, _ = masked_sce_focal_loss(logits2, targets, mask2, 0.1, 0.5, 2.0)
    assert np.isclose(float(ref), float(poisoned))
    assert not np.isclose(float(base), float(ref))


def test_all_masked_is_finite_zero():
    logits, targets, mask = _toy()
    loss, _ = masked_sce_focal_loss(logits, targets, jnp.zeros_like(mask), 0.1, 0.5, 2.0)
    assert float(loss) == 0.0


def test_smoothing_lower_bounds_loss():
    logits = jnp.asarray([[20.0]])
    targets = jnp.asarray([[1.0]])
    mask = jnp.ones_like(targets)
    loss_sm, _ = masked_sce_focal_loss(logits, targets, mask, 0.1, 0.0, 0.0)
    loss_hard, _ = masked_sce_focal_loss(logits, targets, mask, 0.0, 0.0, 0.0)
    assert float(loss_sm) > float(loss_hard)
