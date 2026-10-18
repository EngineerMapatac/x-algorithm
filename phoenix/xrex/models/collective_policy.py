# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import logging
import re
from functools import lru_cache

from jax.interpreters import mlir

rank_logger = logging.getLogger("collective_policy")


@lru_cache(maxsize=1)
def _get_layer_pattern() -> re.Pattern:
    return re.compile(r"decoder_layer(_dense)?_(\d+)")


def _extract_layer_info(ops: set[str]) -> tuple[str | None, int | None]:
    pattern = _get_layer_pattern()
    for op in ops:
        match = pattern.search(op)
        if match:
            layer_type = "dense" if match.group(1) else "regular"
            layer_num = int(match.group(2))
            return layer_type, layer_num
    return None, None


def _dp_allreduce_pipelining_policy(ctx: mlir.LoweringRuleContext):
    attrs = {}
    name_stack = str(ctx.name_stack)
    ops = set(name_stack.split("/"))

    layer_type, layer_num = _extract_layer_info(ops)

    is_first_layer = layer_num is not None and layer_num == 0
    should_pipeline = is_first_layer

    if not should_pipeline:
        attrs["_pipelining"] = "disabled"

    if layer_type is not None and layer_num is not None:
        attrs["_xla_collective_group"] = f"dp_{layer_type}_{layer_num}_ar"

    return attrs


def make_dp_allreduce_policy():
    return _dp_allreduce_pipelining_policy
