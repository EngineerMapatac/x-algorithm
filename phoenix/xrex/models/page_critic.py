# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from __future__ import annotations

from typing import NamedTuple

import haiku as hk
import jax
import jax.numpy as jnp
import numpy as np
import optax
from jax.sharding import PartitionSpec as P

from xai_configlib import Config, configclass
from xrex.models.model_utils import get_parameter


@configclass
class PageCriticConfig(Config):
    hidden_dim: int = 512
    prefix_dropout_period: int = 8
    max_slate_slots: int = 64
    exploration_return_weight: float = 9.0
    return_scale: float = 10.0
    td_lambda: float = 0.9


class PageCriticOutputs(NamedTuple):
    total: jax.Array
    stats: dict[str, jax.Array]
    q_grid_sorted: jax.Array


class PageCriticHeads(hk.Module):
    def __init__(self, config: PageCriticConfig, name: str = "page_critic"):
        super().__init__(name=name)
        self._config = config

    def _hidden(self, emb_size: int) -> tuple[jax.Array, jax.Array]:
        hidden = self._config.hidden_dim
        w_h = get_parameter(
            "critic_hidden_w",
            [3 * emb_size + 1, hidden],
            dtype=jnp.float32,
            init=hk.initializers.VarianceScaling(1.0, mode="fan_in"),
            pspec=P(None, None),
        )
        b_h = get_parameter(
            "critic_hidden_b",
            [hidden],
            dtype=jnp.float32,
            init=hk.initializers.Constant(0.0),
            pspec=P(None),
        )
        return w_h, b_h

    def _out_w(self) -> jax.Array:
        return get_parameter(
            "critic_out_w",
            [self._config.hidden_dim, 2],
            dtype=jnp.float32,
            init=hk.initializers.VarianceScaling(1.0, mode="fan_in"),
            pspec=P(None, None),
        )

    def pre_activation(
        self, emb: jax.Array, slot_feature: jax.Array
    ) -> tuple[jax.Array, jax.Array]:
        d = emb.shape[-1]
        w_h, b_h = self._hidden(d)
        cand = jnp.einsum("bcd,dh->bch", emb.astype(jnp.float32), w_h[:d]) + b_h
        return cand, slot_feature[..., None].astype(jnp.float32) * w_h[d]

    def prefix_hidden(self, emb_sorted: jax.Array, slate_sorted: jax.Array) -> jax.Array:
        d = emb_sorted.shape[-1]
        w_h, _ = self._hidden(d)
        slate_f = slate_sorted.astype(jnp.float32)[:, :, None]
        emb_f = emb_sorted.astype(jnp.float32)
        pm = jnp.einsum("bsd,dh->bsh", emb_f, w_h[d + 1 : 2 * d + 1]) * slate_f
        pl = jnp.einsum("bsd,dh->bsh", emb_f, w_h[2 * d + 1 :]) * slate_f
        count = jnp.cumsum(slate_f, axis=1) - slate_f
        mean_h = (jnp.cumsum(pm, axis=1) - pm) / jnp.maximum(count, 1.0)
        last_h = jnp.concatenate([jnp.zeros_like(pl[:, :1]), pl[:, :-1]], axis=1)
        return mean_h + last_h

    def __call__(self, pre_total: jax.Array) -> tuple[jax.Array, jax.Array]:
        out = jnp.einsum("bch,ho->bco", jax.nn.gelu(pre_total), self._out_w())
        return out[..., 0], out[..., 1]

    def value_grid(
        self,
        cand_hidden: jax.Array,
        step_hidden: jax.Array,
    ) -> jax.Array:
        w_val = self._out_w()[:, 1]
        pre = cand_hidden[:, None] + step_hidden[:, :, None]
        return jnp.einsum("btch,h->btc", jax.nn.gelu(pre), w_val)


def page_critic_value_matrix(
    *, config: PageCriticConfig, candidate_embeddings: jax.Array, top_k: int
) -> jax.Array:
    heads = PageCriticHeads(config)
    slots = np.minimum(np.arange(top_k) / float(config.max_slate_slots), 1.0).astype(np.float32)
    cand, slot = heads.pre_activation(candidate_embeddings, jnp.asarray(slots))
    grid = heads.value_grid(cand, slot[None])
    return (grid * config.return_scale).transpose(0, 2, 1)


def masked_mean(x: jax.Array, mask: jax.Array) -> jax.Array:
    m = mask.astype(jnp.float32)
    return jnp.sum(x * m) / jnp.maximum(jnp.sum(m), 1.0)


def discounted_suffix_sum(x: jax.Array, decay: float, *, strict: bool = False) -> jax.Array:
    lag = np.arange(x.shape[1])[None, :] - np.arange(x.shape[1])[:, None]
    w = np.where(lag > 0 if strict else lag >= 0, decay ** np.maximum(lag, 0), 0.0)
    return (w.astype(np.float32) * x[:, None, :]).sum(axis=-1)


def page_critic_td_target(
    *,
    td_lambda: float,
    gamma: float,
    q_served: jax.Array,
    reward: jax.Array,
    step_mask: jax.Array,
    impressed: jax.Array,
) -> jax.Array:
    impressed = impressed & step_mask
    q_a = jax.lax.stop_gradient(q_served.astype(jnp.float32))
    r = jnp.where(step_mask, reward.astype(jnp.float32), 0.0)
    alive_after = jax.lax.cumsum(impressed.astype(jnp.int32), axis=1, reverse=True) > 0
    alive_next = jnp.pad(alive_after[:, 1:], ((0, 0), (0, 1)))
    a = r + jnp.where(
        alive_next, gamma * (1.0 - td_lambda) * jnp.pad(q_a[:, 1:], ((0, 0), (0, 1))), 0.0
    )
    g = a + discounted_suffix_sum(jnp.where(alive_after, a, 0.0), gamma * td_lambda, strict=True)
    return jax.lax.stop_gradient(jnp.where(step_mask, g, 0.0))


def page_critic_loss(
    *,
    config: PageCriticConfig,
    emb_sorted: jax.Array,
    slate_sorted: jax.Array,
    served_slot_sorted: jax.Array,
    supervised_sorted: jax.Array,
    adv_label_sorted: jax.Array,
    reward_sorted: jax.Array,
    return_sorted: jax.Array,
    explore_page: jax.Array,
    gamma: float,
    grid_steps: int,
) -> PageCriticOutputs:
    S = emb_sorted.shape[1]
    slot_feature = jnp.where(
        slate_sorted,
        jnp.clip(served_slot_sorted.astype(jnp.float32) / config.max_slate_slots, 0.0, 1.0),
        0.0,
    )
    prefix_rows = jnp.arange(emb_sorted.shape[0]) % config.prefix_dropout_period != 0
    pad = ((0, 0), (0, S - grid_steps), (0, S - grid_steps))

    def _critic(name: str) -> tuple[jax.Array, jax.Array, jax.Array]:
        critic = PageCriticHeads(config, name=name)
        cand, slot = critic.pre_activation(emb_sorted, slot_feature)
        prefix = critic.prefix_hidden(emb_sorted, slate_sorted)
        step = slot + prefix * prefix_rows.astype(jnp.float32)[:, None, None]
        cont_logit, value = critic(cand + step)
        grid = critic.value_grid(cand[:, :grid_steps], step[:, :grid_steps])
        return cont_logit, value, jnp.pad(grid * config.return_scale, pad)

    cont_logit, value1, grid1 = _critic("page_critic")
    _, value2, grid2 = _critic("page_critic2")
    cont_loss = masked_mean(
        optax.sigmoid_binary_cross_entropy(cont_logit, adv_label_sorted), supervised_sorted
    )

    target = (
        page_critic_td_target(
            td_lambda=config.td_lambda,
            gamma=gamma,
            q_served=jnp.minimum(value1, value2) * config.return_scale,
            reward=reward_sorted,
            step_mask=slate_sorted,
            impressed=supervised_sorted,
        )
        / config.return_scale
    )
    ret_w = supervised_sorted.astype(jnp.float32) * jnp.where(
        explore_page[:, None], config.exploration_return_weight, 1.0
    )

    def _return_loss(value: jax.Array) -> jax.Array:
        return masked_mean(0.5 * jnp.square(value - target), ret_w)

    ret_loss = _return_loss(value1)
    total = cont_loss + ret_loss + _return_loss(value2)
    g_scaled = return_sorted / config.return_scale

    def _rmse(value: jax.Array) -> jax.Array:
        return jnp.sqrt(masked_mean((value - g_scaled) ** 2, supervised_sorted))

    stats = {
        "page_rl/cont_loss": cont_loss,
        "page_rl/return_loss": ret_loss,
        "page_rl/return_rmse_scaled": _rmse(value1),
        "page_rl/critic2_return_rmse_scaled": _rmse(value2),
    }
    return PageCriticOutputs(total=total, stats=stats, q_grid_sorted=jnp.minimum(grid1, grid2))
