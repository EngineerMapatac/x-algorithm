# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import os

from xrex.configs.xrecsys_gen_recs import CONFIGS as model_cfgs_gen_recs
from xrex.inference import service_registry
from xrex.inference.gen_recs_runner import GenRecsModelRunner
from xrex.models.recsys_gen_recs_model import RecsysGenRecsModelConfig


def _apply_gen_recs_service_env(service_type: str) -> None:
    if service_type == "gen_recs":
        add_flags = os.environ.get("ADD_XLA_FLAGS", "")
        add_flags += " --xla_gpu_enable_triton_gemm=true"
        os.environ["ADD_XLA_FLAGS"] = add_flags


service_registry.EXTRA_MODEL_CFGS.update(model_cfgs_gen_recs)
if RecsysGenRecsModelConfig not in service_registry.EXTRA_CONFIG_TYPES:
    service_registry.EXTRA_CONFIG_TYPES.append(RecsysGenRecsModelConfig)
service_registry.EXTRA_RUNNERS.setdefault("gen_recs", GenRecsModelRunner)
if _apply_gen_recs_service_env not in service_registry.SERVICE_ENV_HOOKS:
    service_registry.SERVICE_ENV_HOOKS.append(_apply_gen_recs_service_env)

assert "gen_recs" in service_registry.EXTRA_RUNNERS, (
    "gen_recs_services: gen_recs runner dispatch missing"
)
assert set(model_cfgs_gen_recs) <= service_registry.EXTRA_MODEL_CFGS.keys(), (
    "gen_recs_services: gen-recs configs missing from EXTRA_MODEL_CFGS"
)
assert RecsysGenRecsModelConfig in service_registry.EXTRA_CONFIG_TYPES, (
    "gen_recs_services: RecsysGenRecsModelConfig missing from EXTRA_CONFIG_TYPES"
)
assert _apply_gen_recs_service_env in service_registry.SERVICE_ENV_HOOKS, (
    "gen_recs_services: per-service env hook not registered"
)
