# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from __future__ import annotations

import logging
import os
import tempfile
from pathlib import Path
from typing import Any

import jax

logger = logging.getLogger(__name__)


def pin_visible_devices(n: int) -> None:
    n = max(1, int(n))
    cur = os.environ.get("CUDA_VISIBLE_DEVICES")
    pin = None
    if cur is None:
        pin = ",".join(str(i) for i in range(n))
        os.environ["CUDA_VISIBLE_DEVICES"] = pin
        logger.info(
            "Single-host launchpad: CUDA_VISIBLE_DEVICES unset -> pinned to %r "
            "(num_devices_per_process=%d)",
            pin,
            n,
        )
    elif cur.strip() != "":
        listed = [d.strip() for d in cur.split(",") if d.strip() != ""]
        if len(listed) > n:
            pin = ",".join(listed[:n])
            os.environ["CUDA_VISIBLE_DEVICES"] = pin
            logger.info(
                "Single-host launchpad: CUDA_VISIBLE_DEVICES=%r -> pinned to %r "
                "(num_devices_per_process=%d)",
                cur,
                pin,
                n,
            )
    if pin is not None:
        try:
            import jax

            jax.config.update("jax_cuda_visible_devices", pin)
        except Exception:
            pass


def _build_trainer_context(run_config: Any) -> Any:
    from xrex.train.trainer import TrainerContext
    from xrex.utils import metadata

    ckpt_cfg = getattr(run_config, "checkpoint_config", None)
    ckpt_dir = getattr(ckpt_cfg, "checkpoint_dir", None) if ckpt_cfg is not None else None
    if not ckpt_dir:
        ckpt_dir = tempfile.mkdtemp(prefix="phoenix-oss-ckpt-")

    meta = metadata.build_metadata_manager(ckpt_dir)

    run_name = getattr(run_config, "name", "phoenix")
    run, checkpoint = meta.init_run(run_name, run_config, load=getattr(run_config, "load", None))

    run_dir = Path(ckpt_dir) / "run"
    run_dir.mkdir(parents=True, exist_ok=True)

    ctx = TrainerContext(
        rank=0,
        world_size=1,
        hostnames=["localhost"],
        ip_addrs=["127.0.0.1"],
        config=run_config,
        run=run,
        run_dir=run_dir,
        checkpoint=checkpoint,
        meta=meta,
    )
    ctx.coordinator_port = 12355

    return ctx


def run_driver_train(driver_config: Any, run_config: Any) -> None:
    del driver_config
    os.environ.setdefault("XAI_RUN_NAME", getattr(run_config, "name", "phoenix"))

    ctx = _build_trainer_context(run_config)
    logger.info("Built TrainerContext (rank=0, world_size=1, single-host)")

    if getattr(run_config, "enable_metrics_file", False):
        from xrex.driver.hooks import HookManager, MetricsFileHook

        metrics_path = ctx.run_dir / "metrics.jsonl"
        hooks = HookManager()
        hooks.add_hook(MetricsFileHook(path=metrics_path))
        ctx.hooks = hooks
        logger.info("enable_metrics_file: writing per-step metrics to %s", metrics_path)
    pc = getattr(run_config, "parallel_config", None)
    assert pc is not None, (
        f"{type(run_config).__name__} has no parallel_config; the single-host "
        "mesh cannot be resolved"
    )
    requested = int(pc.num_devices_per_process)
    assert requested >= 1, f"parallel_config.num_devices_per_process must be >= 1, got {requested}"
    n = min(requested, jax.device_count())
    if n != requested:
        logger.warning(
            "num_devices_per_process=%d but only %d JAX device(s) are visible; "
            "clamping the mesh to %d. Check CUDA_VISIBLE_DEVICES / the pin.",
            requested,
            jax.device_count(),
            n,
        )
    pc.dp = 1
    pc.ep = n
    pc.num_devices_per_process = n
    pc.num_devices_per_node = n
    if hasattr(run_config, "inference_batch_size"):
        run_config.inference_batch_size = int(getattr(run_config, "bs_per_device", 1) or 1) * n
    logger.info("Forced single-host MESH (%d device(s), ep=%d): %r", n, n, pc)
    logger.info(
        "trainer.bs_per_device=%s  inference_batch_size=%s",
        getattr(run_config, "bs_per_device", "?"),
        getattr(run_config, "inference_batch_size", "?"),
    )

    try:
        run_config.initialize(ctx)
    except Exception:
        logger.exception("trainer.initialize(ctx) failed")
        raise

    try:
        run_config.run()
    finally:
        if ctx.hooks is not None:
            ctx.hooks.on_train_stop()
        try:
            jax.distributed.shutdown()
        except Exception:
            pass
