# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import logging
import subprocess

logger = logging.getLogger(__name__)


def nsys_start(run_dir: str, session_name: str, options: str):
    output = f"{run_dir}/nsys-start-profile.nsys-rep"
    args = [
        "nsys",
        "start",
        f"--output={output}",
        f"--session={session_name}",
    ] + (options.split(",") if options else [])

    logger.info(f"nsys start, command: {args}")
    subprocess.run(args, check=False)


def nsys_stop_and_shutdown(session_name: str):
    if session_name:
        logger.info(f"Stopping nsys profile on session {session_name}")
        subprocess.run(["nsys", "stop", f"--session={session_name}"], check=False)
        logger.info(f"Shutting down nsys on session {session_name}")
        subprocess.run(["nsys", "shutdown", f"--session={session_name}"], check=False)
