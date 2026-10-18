from __future__ import annotations

import numpy as np


def average_precision(y_true: np.ndarray, scores: np.ndarray) -> float:
    order = np.argsort(-scores, kind="stable")
    y = y_true[order].astype(np.float64)
    n_pos = y.sum()
    if n_pos == 0:
        return float("nan")
    tp = np.cumsum(y)
    precision = tp / np.arange(1, len(y) + 1)
    return float((precision * y).sum() / n_pos)
