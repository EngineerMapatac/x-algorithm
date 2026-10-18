# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from xrex.configs.xrecsys_sid_retrieval import CONFIGS as model_cfgs_sid_retrieval
from xrex.inference import service_registry
from xrex.inference.sid_retrieval_runner import SidRetrievalModelRunner
from xrex.models.recsys_sid_retrieval_model import RecsysSIDRetrievalConfig

service_registry.EXTRA_MODEL_CFGS.update(model_cfgs_sid_retrieval)
if RecsysSIDRetrievalConfig not in service_registry.EXTRA_CONFIG_TYPES:
    service_registry.EXTRA_CONFIG_TYPES.append(RecsysSIDRetrievalConfig)
service_registry.EXTRA_RUNNERS.setdefault("sid_retrieval", SidRetrievalModelRunner)
if "sid_retrieval" not in service_registry.RETRIEVAL_DATASET_SERVICE_TYPES:
    service_registry.RETRIEVAL_DATASET_SERVICE_TYPES.append("sid_retrieval")

assert "sid_retrieval" in service_registry.EXTRA_RUNNERS, (
    "sid_services: sid_retrieval runner dispatch missing"
)
assert set(model_cfgs_sid_retrieval) <= service_registry.EXTRA_MODEL_CFGS.keys(), (
    "sid_services: SID-retrieval configs missing from EXTRA_MODEL_CFGS"
)
assert RecsysSIDRetrievalConfig in service_registry.EXTRA_CONFIG_TYPES, (
    "sid_services: RecsysSIDRetrievalConfig missing from EXTRA_CONFIG_TYPES"
)
assert "sid_retrieval" in service_registry.RETRIEVAL_DATASET_SERVICE_TYPES, (
    "sid_services: sid_retrieval missing from RETRIEVAL_DATASET_SERVICE_TYPES"
)
