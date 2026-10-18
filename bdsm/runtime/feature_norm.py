from __future__ import annotations

import numpy as np

_GROUP_ASSIGNMENTS: dict[int, int] = {
    1: 1,
    2: 1,
    4: 2,
    7: 2,
    6: 3,
    9: 3,
    5: 3,
    8: 3,
    29: 4,
    30: 4,
    14: 4,
    70: 4,
    60: 4,
    39: 4,
    11: 5,
    12: 5,
    31: 6,
    37: 6,
    36: 7,
    34: 7,
    21: 8,
    61: 8,
    62: 8,
    10: 9,
    16: 9,
    67: 9,
    23: 10,
    63: 10,
    64: 10,
    25: 11,
    65: 11,
    66: 11,
    50: 12,
    53: 12,
    54: 12,
    55: 12,
    56: 12,
    58: 12,
    13: 12,
}

ACTION_GROUP_TABLE = np.zeros(256, dtype=np.int32)
for _action_id, _group in _GROUP_ASSIGNMENTS.items():
    ACTION_GROUP_TABLE[_action_id] = _group


def remap_client_platform(app_id: np.ndarray) -> np.ndarray:
    return np.where(
        app_id == 258901,
        1,
        np.where(
            app_id == 129032,
            2,
            np.where(app_id == 3033300, 3, np.where((app_id > 0) & (app_id < 1000), 4, 0)),
        ),
    ).astype(np.int32)


def normalize_aseq(aseq: dict[str, np.ndarray]) -> dict[str, np.ndarray]:
    out = dict(aseq)
    action_ids = np.argmax(out["action_types"], axis=-1)
    out["action_group"] = ACTION_GROUP_TABLE[action_ids]
    if "client_app_id" in out:
        out["client_platform"] = remap_client_platform(out["client_app_id"])
    return out
