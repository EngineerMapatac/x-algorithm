from __future__ import annotations

import hashlib
import json

import numpy as np


def load_params(
    npz_path: str, manifest_path: str | None = None, verify_hashes: bool = False
) -> dict:
    z = np.load(npz_path)
    flat = {key: z[key] for key in z.files}

    if manifest_path is not None:
        with open(manifest_path) as f:
            manifest = json.load(f)
        entries = manifest["params"]
        if set(entries) != set(flat):
            missing = sorted(set(entries) - set(flat))
            extra = sorted(set(flat) - set(entries))
            raise ValueError(f"key mismatch vs manifest: missing={missing} extra={extra}")
        for key, arr in flat.items():
            meta = entries[key]
            if list(arr.shape) != meta["shape"] or str(arr.dtype) != meta["dtype"]:
                raise ValueError(
                    f"{key}: got {arr.shape}/{arr.dtype}, "
                    f"manifest says {meta['shape']}/{meta['dtype']}"
                )
            if verify_hashes:
                digest = hashlib.sha256(np.ascontiguousarray(arr).tobytes()).hexdigest()
                if digest != meta["sha256"]:
                    raise ValueError(f"{key}: sha256 mismatch")

    params: dict[str, dict[str, np.ndarray]] = {}
    for key, arr in flat.items():
        module, _, name = key.partition("/")
        params.setdefault(module, {})[name] = arr
    return params
