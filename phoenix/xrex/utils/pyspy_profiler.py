# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import logging
import os
import signal
import subprocess
import threading
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path

logger = logging.getLogger(__name__)


@dataclass
class _PyspyState:
    process: subprocess.Popen | None = None
    lock: threading.Lock = field(default_factory=threading.Lock)

    def reset(self) -> None:
        self.process = None


_pyspy_state = _PyspyState()


def start_pyspy_trace(dir_path: os.PathLike, blocking: bool = False, rate: int = 100) -> None:
    with _pyspy_state.lock:
        if _pyspy_state.process is not None:
            raise RuntimeError("Only one py-spy profile may be run at a time.")

        timestamp = datetime.now().strftime("%Y-%m-%dT%H:%M:%SZ")
        pid = os.getpid()
        output_path = Path(dir_path) / f"{pid}-{timestamp}.json"
        output_path.parent.mkdir(parents=True, exist_ok=True)
        logger.info(f"Profiling to {output_path}...")

        cmd = [
            "py-spy",
            "record",
            "--pid",
            str(pid),
            "--format",
            "speedscope",
            "--nonblocking" if not blocking else "--native",
            "--rate",
            f"{rate}",
            "-o",
            str(output_path),
        ]

        env = os.environ.copy()
        env["RUST_LOG"] = "warn"

        process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
        _pyspy_state.process = process


def stop_pyspy_trace() -> None:
    with _pyspy_state.lock:
        if _pyspy_state.process is None:
            raise RuntimeError("No py-spy profile started")

        _pyspy_state.process.send_signal(signal.SIGINT)

        stdout, stderr = _pyspy_state.process.communicate()

        return_code = _pyspy_state.process.returncode
        if return_code != 0:
            logger.error(
                f"py-spy profiling failed with return code {return_code}. Stdout: {stdout}. Stderr: {stderr}."
            )
        logger.info(f"py-spy profiling stdout: {stdout}.")

        _pyspy_state.reset()
