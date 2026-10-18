# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from __future__ import annotations

import json
import logging
import os
import threading
import urllib.parse
import urllib.request

from xrex import settings
from xrex.utils import cluster as cluster_identity

logger = logging.getLogger(__name__)

_API_BASE = settings.RUN_TRACKER_URL

_CLIENT_ID = "xrex-run-notify"
_CHECKPOINT_PATH_PREFIX = settings.CHECKPOINT_PATH_PREFIX


def _post(kind: str, body_key: str, path: str, required_prefix: str) -> None:
    if not _API_BASE:
        return
    if not str(path).startswith(required_prefix):
        return
    cluster = cluster_identity.get_cluster()
    if not cluster:
        return
    url = f"{_API_BASE}/api/training-runs/notify/{kind}?cluster={urllib.parse.quote(cluster)}"
    data = json.dumps({body_key: str(path)}).encode()

    def _fire() -> None:
        try:
            req = urllib.request.Request(
                url,
                data=data,
                method="POST",
                headers={
                    "Content-Type": "application/json",
                    "X-Nebula-Client": _CLIENT_ID,
                },
            )
            urllib.request.urlopen(req, timeout=5).read()
        except Exception as exc:
            logger.warning("toolbox_notify %s failed: %s", kind, exc)

    threading.Thread(target=_fire, name=f"toolbox-notify-{kind}", daemon=True).start()


def notify_checkpoint_async(checkpoint_path: str | os.PathLike[str]) -> None:
    _post(
        "checkpoint",
        "checkpointPath",
        os.fspath(checkpoint_path),
        required_prefix=_CHECKPOINT_PATH_PREFIX,
    )
