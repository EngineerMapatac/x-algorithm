# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from xrex.configs.xrecsys_reply_two_tower_v2 import CONFIGS as model_cfgs_reply_two_tower_v2
from xrex.inference import service_registry

service_registry.EXTRA_MODEL_CFGS.update(model_cfgs_reply_two_tower_v2)

assert set(model_cfgs_reply_two_tower_v2) <= service_registry.EXTRA_MODEL_CFGS.keys(), (
    "reply_services: reply-ranker configs missing from EXTRA_MODEL_CFGS"
)
