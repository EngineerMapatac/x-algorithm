from __future__ import annotations

import time


class AdaptiveMinFav:
    def __init__(
        self,
        start: int = 20_000,
        *,
        target_qps: float = 5.0,
        gain: float = 0.3,
        deadband: float = 0.1,
        window_s: float = 30.0,
        floor: int = 0,
        cap: int = 20_000,
        warmup_s: float = 60.0,
        clock=time.monotonic,
    ) -> None:
        if target_qps <= 0:
            raise ValueError("target_qps must be > 0")
        self.min_fav = start
        self.target_qps = target_qps
        self.gain = gain
        self.deadband = deadband
        self.window_s = window_s
        self.floor = floor
        self.cap = cap
        self.warmup_s = warmup_s
        self.last_qps: float | None = None
        self._clock = clock
        self._started = clock()
        self._window_start = self._started
        self._passes = 0

    def observe_pass(self) -> None:
        self._passes += 1

    def threshold(self) -> int:
        self._roll_window()
        return self.min_fav

    def _roll_window(self) -> None:
        now = self._clock()
        elapsed = now - self._window_start
        if elapsed < self.window_s:
            return
        qps = self._passes / elapsed
        self.last_qps = qps
        self._passes = 0
        self._window_start = now
        if now - self._started < self.warmup_s:
            return
        error = (qps - self.target_qps) / self.target_qps
        if abs(error) <= self.deadband:
            return
        error = max(-1.0, min(1.0, error))
        scaled = self.min_fav * (1.0 + self.gain * error)
        if error > 0:
            new = max(int(scaled), self.min_fav + 1)
        else:
            new = min(int(scaled), self.min_fav - 1)
        self.min_fav = max(self.floor, min(self.cap, new))
