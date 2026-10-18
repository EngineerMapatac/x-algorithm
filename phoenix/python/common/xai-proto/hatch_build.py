# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import os
import pathlib
import subprocess
import sys
from pathlib import Path

from hatchling.builders.hooks.plugin.interface import BuildHookInterface

PARENT_DIR = "."

PROTOS = [
    "proto/recsys.proto",
    "proto/copy.proto",
    "proto/recsys_kafka.proto",
]


class CustomBuildHook(BuildHookInterface):
    def initialize(self, version, build_data):
        root = Path(self.root)
        out_dir = root / "xai_proto"
        out_dir.mkdir(parents=True, exist_ok=True)
        (out_dir / "__init__.py").write_text("")
        run_protoc(out_dir)
        force_include = {}
        for pattern in ("*_pb2.py", "*_pb2.pyi", "*_pb2_grpc.py"):
            for f in out_dir.glob(pattern):
                force_include[str(f)] = f"xai_proto/{f.name}"
        build_data["force_include"] = force_include


def run_protoc(out_dir: Path) -> None:
    skip_missing = os.environ.get("XAI_PROTO_SKIP_MISSING", "") == "1"
    xai_root = pathlib.Path(__file__).parent / PARENT_DIR
    includes = []
    protos = []
    for proto in PROTOS:
        p = xai_root / proto
        if skip_missing and not p.exists():
            print(
                f"xai-proto hatch_build: SKIPPING {proto} (absent; XAI_PROTO_SKIP_MISSING=1)",
                file=sys.stderr,
            )
            continue
        includes.append(f"-I{p.parent}")
        protos.append(str(p))

    subprocess.check_call(
        [
            sys.executable,
            "-m",
            "grpc_tools.protoc",
            *includes,
            f"--python_out={out_dir}",
            f"--pyi_out={out_dir}",
            f"--grpc_python_out={out_dir}",
            *protos,
        ]
    )

    sed = [
        "sed",
        "-i",
        r"s/^import\(.*_pb2\)/from . import\1/g",
        *out_dir.glob("*_pb2.py"),
        *out_dir.glob("*_grpc.py"),
    ]
    if not is_gnu_sed():
        sed.insert(2, "")
    subprocess.check_call(sed)


def is_gnu_sed():
    try:
        output = subprocess.check_output(["sed", "--version"], stderr=subprocess.STDOUT)
        return "GNU" in output.decode()
    except subprocess.CalledProcessError:
        return False
