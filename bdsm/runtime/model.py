from __future__ import annotations

import math
from dataclasses import dataclass

import haiku2 as hk
import jax
import jax.numpy as jnp


@dataclass(frozen=True)
class BackboneConfig:
    emb_size: int
    num_layers: int
    query_heads: int
    kv_heads: int
    ff_mult: int
    n_action_types: int
    seq_len: int
    contrastive_dim: int = 128
    rope_base: float = 10000.0

    @classmethod
    def from_checkpoint_config(cls, cfg: dict) -> BackboneConfig:
        return cls(
            emb_size=cfg["emb_size"],
            num_layers=cfg["num_layers"],
            query_heads=cfg["query_heads"],
            kv_heads=cfg["kv_heads"],
            ff_mult=cfg["ff_mult"],
            n_action_types=cfg["n_action_types"],
            seq_len=cfg["seq_len"],
        )


def rms_norm(x: jax.Array, name: str) -> jax.Array:
    scale = hk.get_parameter(f"{name}/scale", [x.shape[-1]], init=jnp.ones)
    ms = jnp.mean(x**2, axis=-1, keepdims=True)
    return x * scale * jax.lax.rsqrt(ms + 1e-6)


def _rotate_half(x: jax.Array) -> jax.Array:
    x1 = x[..., : x.shape[-1] // 2]
    x2 = x[..., x.shape[-1] // 2 :]
    return jnp.concatenate([-x2, x1], axis=-1)


def _apply_rotary_pos_emb(
    q: jax.Array,
    k: jax.Array,
    cos: jax.Array,
    sin: jax.Array,
) -> tuple[jax.Array, jax.Array]:
    if cos.ndim == 3:
        cos = cos[:, :, None, :]
        sin = sin[:, :, None, :]
    q_embed = (q * cos) + (_rotate_half(q) * sin)
    k_embed = (k * cos) + (_rotate_half(k) * sin)
    return q_embed, k_embed


def build_time_aware_rope(
    impr_ts: jax.Array,
    head_dim: int,
    base: float,
) -> tuple[jax.Array, jax.Array]:
    B, S = impr_ts.shape
    dim_indices = jnp.arange(0, head_dim, 2, dtype=jnp.float32)
    inv_freq = 1.0 / (base ** (dim_indices / head_dim))

    ts_f = impr_ts.astype(jnp.float32)
    ts_min = jnp.where(ts_f > 0, ts_f, jnp.float32(1e12))
    ts_min = jnp.min(ts_min, axis=1, keepdims=True)
    ts_norm = jnp.where(ts_f > 0, ts_f - ts_min, 0.0)

    positions = jnp.concatenate([jnp.zeros((B, 1)), ts_norm], axis=1)
    theta = positions[:, :, None] * inv_freq[None, None, :]
    cos = jnp.concatenate([jnp.cos(theta), jnp.cos(theta)], axis=-1)
    sin = jnp.concatenate([jnp.sin(theta), jnp.sin(theta)], axis=-1)
    return cos, sin


def _repeat_kv(x: jax.Array, n_rep: int) -> jax.Array:
    if n_rep == 1:
        return x
    B, T, kv_heads, Dh = x.shape
    x = jnp.broadcast_to(x[:, :, :, None, :], (B, T, kv_heads, n_rep, Dh))
    return x.reshape(B, T, kv_heads * n_rep, Dh)


class Backbone(hk.Module):
    def __init__(self, config: BackboneConfig, name: str = "abuse_v3"):
        super().__init__(name=name)
        self.c = config
        self.D = config.emb_size

    def _build_input_embeddings(self, batch: dict) -> tuple[jax.Array, jax.Array]:
        D = self.D
        aseq = batch["action_seq"]
        B = aseq["padding_mask"].shape[0]
        init = hk.initializers.TruncatedNormal(stddev=0.02)
        vinit = hk.initializers.VarianceScaling(1.0, "fan_out", "truncated_normal")

        action_table = hk.get_parameter(
            "action_emb",
            [self.c.n_action_types, D],
            init=vinit,
        )
        action_idx = jnp.argmax(aseq["action_types"], axis=-1)
        action_emb = action_table[jnp.clip(action_idx, 0, self.c.n_action_types - 1)]

        def semb(name: str, vocab: int, dim: int, feat: jax.Array) -> jax.Array:
            t = hk.get_parameter(f"{name}_emb", [vocab, dim], init=init)
            return t[jnp.clip(feat, 0, vocab - 1)]

        day_of_week = ((aseq["impr_ts"] // 86400) + 4) % 7

        discrete_parts = [
            semb("action_group", 14, 16, aseq["action_group"]),
            semb("action_category", 3, 8, aseq["action_category"]),
            semb("client_platform", 5, 8, aseq["client_platform"]),
            semb(
                "client_app",
                512,
                32,
                aseq.get("client_app_id", jnp.zeros_like(aseq["action_group"])) % 512,
            ),
            semb("page_context", 128, 32, aseq["page_context"]),
            semb("country_code", 256, 32, aseq["country_code"]),
            semb("timezone", 32, 16, aseq["timezone"]),
            semb("post_language", 64, 16, aseq["post_language"]),
            semb("device_network", 8, 8, aseq["device_network"]),
            semb("search_query", 256, 32, aseq["search_query_hash"]),
            semb("section", 256, 32, aseq["section_hash"]),
            semb("transition", 8192, 64, aseq["action_transition"] % 8192),
            semb("hour_of_day", 24, 16, aseq["hour_of_day"]),
            semb("day_of_week", 7, 8, day_of_week),
            semb("product_surface", 8, 16, aseq["product_surface"]),
        ]
        discrete_concat = jnp.concatenate(discrete_parts, axis=-1)
        concat_dim = sum(p.shape[-1] for p in discrete_parts)
        proj_mat = hk.get_parameter("discrete_proj", [concat_dim, D], init=vinit)
        x = action_emb + jnp.dot(discrete_concat, proj_mat)

        def cont(name: str, val: jax.Array) -> jax.Array:
            w1 = hk.get_parameter(f"{name}_w1", [1, 64], init=init)
            w2 = hk.get_parameter(f"{name}_w2", [64, D], init=init)
            h = jax.nn.gelu(jnp.dot(val[:, :, None], w1))
            return jnp.dot(h, w2)

        x = x + cont("battery", aseq["device_battery"])
        x = x + cont("brightness", aseq["device_brightness"])
        x = x + cont("storage", aseq["device_storage_ratio"])
        x = x + cont("dwell_ratio", aseq["dwell_to_action_ratio"])
        x = x + cont("serve2act", aseq["serve_to_action_ms"])
        x = x + cont("popularity", aseq["target_popularity"])
        x = x + cont("view_count", aseq["view_count"])
        x = x + cont("video_dur", aseq["video_duration_ms"])
        x = x + cont("ext_link", aseq["external_link_duration"])
        x = x + cont("tweet_age", aseq["time_since_tweet_created"])
        x = x + cont("burst", aseq["action_burst_count"].astype(jnp.float32) / 10.0)
        x = x + cont("streak", aseq["same_author_streak"].astype(jnp.float32) / 100.0)
        x = x + cont("feed_pos", aseq["tweet_feed_position"].astype(jnp.float32) / 50.0)

        bools = jnp.stack(
            [
                aseq["is_following_author"].astype(jnp.float32),
                aseq["target_has_media"].astype(jnp.float32),
                aseq["is_promoted_tweet"].astype(jnp.float32),
                aseq["device_charging"].astype(jnp.float32),
                aseq["engagement_without_impression"].astype(jnp.float32),
                aseq["had_prior_render"].astype(jnp.float32),
            ],
            axis=-1,
        )
        wb1 = hk.get_parameter("bool_w1", [6, 64], init=init)
        wb2 = hk.get_parameter("bool_w2", [64, D], init=init)
        x = x + jnp.dot(jax.nn.gelu(jnp.dot(bools, wb1)), wb2)

        zero = jnp.zeros_like(aseq["action_entropy"])
        stats = jnp.stack(
            [
                aseq["action_entropy"],
                aseq["timing_regularity"],
                aseq["dwell_fraction"],
                aseq["unique_ips"],
                aseq["sequence_time_span"],
                aseq["target_tweet_repetition"],
                aseq.get("entropy_h1", zero),
                aseq.get("regularity_h1", zero),
                aseq.get("dwell_fraction_h1", zero),
                aseq.get("unique_ips_h1", zero),
                aseq.get("time_span_h1", zero),
                aseq.get("tweet_rep_h1", zero),
                aseq.get("entropy_h2", zero),
                aseq.get("regularity_h2", zero),
                aseq.get("dwell_fraction_h2", zero),
                aseq.get("unique_ips_h2", zero),
                aseq.get("time_span_h2", zero),
                aseq.get("tweet_rep_h2", zero),
            ],
            axis=-1,
        ).astype(jnp.float32)
        ws1 = hk.get_parameter("stats_w1", [18, 64], init=init)
        ws2 = hk.get_parameter("stats_w2", [64, D], init=init)
        x = x + jnp.dot(jax.nn.gelu(jnp.dot(stats, ws1)), ws2)

        x = x + cont("entity_rep", aseq.get("entity_rep_count", zero) / 50.0)

        def multi_emb(name: str, vocab: int, feat: jax.Array) -> jax.Array:
            h0 = jnp.clip(feat, 0, vocab - 1)
            h1 = jnp.clip((feat * 1103515245 + 104729) % vocab, 0, vocab - 1)
            h2 = jnp.clip((feat * 40503 + 12345) % vocab, 0, vocab - 1)
            t0 = hk.get_parameter(f"{name}_emb_0", [vocab, D], init=init)
            t1 = hk.get_parameter(f"{name}_emb_1", [vocab, D], init=init)
            t2 = hk.get_parameter(f"{name}_emb_2", [vocab, D], init=init)
            return t0[h0] + t1[h1] + t2[h2]

        x = x + multi_emb("ip_hash", 1024, aseq["ip_hash"])
        x = x + multi_emb("quoting_author", 4096, aseq["quoting_author_hash"])
        x = x + multi_emb("reply_to_author", 4096, aseq["in_reply_to_author_hash"])
        x = x + multi_emb("rt_author", 4096, aseq["retweeting_author_hash"])
        x = x + multi_emb("quoted_author", 4096, aseq["quoted_author_hash"])

        cont_act = aseq.get("continuous_actions")
        if cont_act is not None:
            x = x + cont("time_delta", cont_act[:, :, 0])
            x = x + cont("dwell_norm", cont_act[:, :, 1])

        cls_emb = hk.get_parameter("cls_emb", [1, 1, D], init=init)
        cls_emb = jnp.broadcast_to(cls_emb, (B, 1, D))
        full_emb = jnp.concatenate([cls_emb, x], axis=1)

        cls_mask = jnp.ones((B, 1), dtype=jnp.bool_)
        full_mask = jnp.concatenate([cls_mask, aseq["padding_mask"]], axis=1)

        return full_emb.astype(jnp.bfloat16), full_mask

    def _transformer_layer(
        self,
        x: jax.Array,
        mask: jax.Array,
        rope_cos: jax.Array,
        rope_sin: jax.Array,
        li: int,
    ) -> jax.Array:
        D = self.D
        H = self.c.query_heads
        KV_H = self.c.kv_heads
        head_dim = D // H
        n_rep = H // KV_H
        vinit = hk.initializers.VarianceScaling(1.0, "fan_in", "truncated_normal")

        nx = rms_norm(x, f"l{li}_an")

        w_q = hk.get_parameter(f"l{li}_a/wq", [D, H, head_dim], init=vinit)
        w_k = hk.get_parameter(f"l{li}_a/wk", [D, KV_H, head_dim], init=vinit)
        w_v = hk.get_parameter(f"l{li}_a/wv", [D, KV_H, head_dim], init=vinit)
        w_o = hk.get_parameter(f"l{li}_a/wo", [H, head_dim, D], init=vinit)

        q = jnp.einsum("btd,dhk->bthk", nx, w_q)
        k = jnp.einsum("btd,dhk->bthk", nx, w_k)
        v = jnp.einsum("btd,dhk->bthk", nx, w_v)

        q, k = _apply_rotary_pos_emb(q, k, rope_cos, rope_sin)

        k = _repeat_kv(k, n_rep)
        v = _repeat_kv(v, n_rep)
        scale = 1.0 / math.sqrt(head_dim)

        q = q.astype(jnp.bfloat16)
        k = k.astype(jnp.bfloat16)
        v = v.astype(jnp.bfloat16)
        bias = jnp.where(mask[:, None, None, :], jnp.bfloat16(0.0), jnp.bfloat16(-1e4))
        out = jax.nn.dot_product_attention(q, k, v, bias=bias, scale=scale)
        out = jnp.einsum("bqhd,hdo->bqo", out.astype(jnp.float32), w_o)
        x = x + out

        nx2 = rms_norm(x, f"l{li}_fn")
        ff_dim = D * self.c.ff_mult
        w_gate = hk.get_parameter(f"l{li}_f/wg", [D, ff_dim], init=vinit)
        w_up = hk.get_parameter(f"l{li}_f/wu", [D, ff_dim], init=vinit)
        w_down = hk.get_parameter(f"l{li}_f/wd", [ff_dim, D], init=vinit)

        gate = jax.nn.silu(jnp.dot(nx2, w_gate))
        up = jnp.dot(nx2, w_up)
        ff_out = jnp.dot(gate * up, w_down)
        x = x + ff_out.astype(x.dtype)

        return x

    def _transformer(
        self,
        x: jax.Array,
        mask: jax.Array,
        rope_cos: jax.Array,
        rope_sin: jax.Array,
    ) -> jax.Array:
        for li in range(self.c.num_layers):
            x = self._transformer_layer(x, mask, rope_cos, rope_sin, li)
        return rms_norm(x, "final_norm")

    def encode(self, batch: dict) -> tuple[jax.Array, jax.Array, jax.Array]:
        D = self.D
        embeddings, mask = self._build_input_embeddings(batch)

        head_dim = D // self.c.query_heads
        rope_cos, rope_sin = build_time_aware_rope(
            batch["action_seq"]["impr_ts"],
            head_dim,
            self.c.rope_base,
        )
        x = self._transformer(embeddings, mask, rope_cos, rope_sin)

        cls_raw = x[:, 0, :].astype(jnp.float32)
        hidden = x[:, 1:, :].astype(jnp.float32)

        C = self.c.contrastive_dim
        init_c = hk.initializers.TruncatedNormal(stddev=0.02)
        w_c1 = hk.get_parameter("contrast/w1", [D, 256], init=init_c)
        w_c2 = hk.get_parameter("contrast/w2", [256, C], init=init_c)
        h_c = jax.nn.gelu(jnp.dot(cls_raw, w_c1))
        cls_embed = jnp.dot(h_c, w_c2)
        cls_embed = cls_embed / (jnp.linalg.norm(cls_embed, axis=-1, keepdims=True) + 1e-8)

        return hidden, cls_raw, cls_embed


def build_encode_fn(config: BackboneConfig):
    @hk.transform
    def encode_fn(batch: dict):
        return Backbone(config).encode(batch)

    return encode_fn
