# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import logging
import threading
import time
from dataclasses import dataclass, field
from os import PathLike
from pathlib import Path

import jax

try:
    from jax._src.lib import _profiler
except ImportError:
    from jax._src.lib import xla_client

    _profiler = xla_client.profiler

logger = logging.getLogger(__name__)
_profiler_server: _profiler.ProfilerServer | None = None
_STOP_TRACE_TIMEOUT_SECS = 20


@dataclass
class _ProfileSession:
    inner: _profiler.ProfilerSession
    log_dir: Path


@dataclass
class _ProfileState:
    session: _ProfileSession | None = None
    lock: threading.Lock = field(default_factory=threading.Lock)

    def reset(self) -> None:
        self.session = None


_profile_state = _ProfileState()


def start_trace(log_dir: str | PathLike[str]) -> None:
    with _profile_state.lock:
        if _profile_state.session is not None:
            raise RuntimeError("Only one profile may be run at a time.")
        _profile_state.session = _ProfileSession(_profiler.ProfilerSession(), Path(log_dir))


def _export_and_write_fdo(sess: _profiler.ProfilerSession, log_dir: Path, xplane: object) -> None:
    t0 = time.monotonic()
    try:
        sess.export(xplane, str(log_dir))
        fdo_path = log_dir / "fdo.pbtxt"
        log = _profiler.get_fdo_profile(xplane, True)
        if isinstance(log, bytes):
            log = log.decode("utf-8")
        fdo_path.write_text(log)
        elapsed = time.monotonic() - t0
        logger.info("Profile exported and FDO written to %s in %.1f s.", fdo_path, elapsed)
    except Exception:
        logger.exception("Failed to export profile / write FDO to %s.", log_dir)


def stop_trace(timeout: float = _STOP_TRACE_TIMEOUT_SECS) -> None:
    logger.info("Stopping trace")
    with _profile_state.lock:
        if _profile_state.session is None:
            raise RuntimeError("No profile started")
        session = _profile_state.session
        sess = session.inner
        xplane = sess.stop()
        thread = threading.Thread(
            target=_export_and_write_fdo,
            args=(sess, session.log_dir, xplane),
            name="export-and-write-fdo",
            daemon=True,
        )
        thread.start()
        _profile_state.reset()

    thread.join(timeout=timeout)
    if thread.is_alive():
        logger.warning(
            "Profile export timed out after %.1fs. Thread is still running in the background.",
            timeout,
        )


def read_fdo_profile(fun_name: str, fdo_dir: str | PathLike[str] | None) -> str | None:
    if fdo_dir is None:
        return None

    fdo_dir = Path(fdo_dir)
    if not fdo_dir.is_dir():
        return None

    file_base = fun_name + ".pbtxt"
    fdo_file_path = fdo_dir / file_base
    if not fdo_file_path.is_file():
        fdo_file_path = fdo_dir / "fdo.pbtxt"
        if not fdo_file_path.is_file():
            return None

    return fdo_file_path.read_text()


def start_profiler_server(port: int, process_index: int) -> _profiler.ProfilerServer:
    if "xai" not in jax.__version__:
        return _profiler.start_server(port)

    global _profiler_server
    if _profiler_server is not None:
        raise ValueError("Only one profiler server can be active at a time.")

    _profiler_server = _profiler.start_server(port, process_index)
    return _profiler_server
