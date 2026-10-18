#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.

from __future__ import annotations

import argparse
import sys
from concurrent import futures
from pathlib import Path

import grpc

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "_sid_proto"))

import sid_lookup_pb2
import sid_lookup_pb2_grpc
from oss_recsys_synth import sid_codes_for_posts


class SidLookupServicer(sid_lookup_pb2_grpc.SidLookupServiceServicer):
    def __init__(self, num_levels: int, codebook_size: int, mm_dim: int, seed: int):
        self.num_levels = num_levels
        self.codebook_size = codebook_size
        self.mm_dim = mm_dim
        self.seed = seed

    def LookupSids(self, request, context):
        post_ids = list(request.post_ids)
        results = []
        if not post_ids:
            return sid_lookup_pb2.LookupSidsResponse(results=results)
        import numpy as np

        ids = np.asarray(post_ids, dtype=np.int64)
        codes = sid_codes_for_posts(
            ids,
            self.num_levels,
            self.codebook_size,
            mm_dim=self.mm_dim,
            seed=self.seed,
        )
        for i, pid in enumerate(post_ids):
            if pid <= 0:
                results.append(sid_lookup_pb2.PostSids(codes=[]))
            else:
                results.append(sid_lookup_pb2.PostSids(codes=[int(c) for c in codes[i]]))
        return sid_lookup_pb2.LookupSidsResponse(results=results)


def serve(port: int, num_levels: int, codebook_size: int, mm_dim: int, seed: int) -> None:
    server = grpc.server(
        futures.ThreadPoolExecutor(max_workers=4),
        options=(("grpc.so_reuseport", 0),),
    )
    sid_lookup_pb2_grpc.add_SidLookupServiceServicer_to_server(
        SidLookupServicer(num_levels, codebook_size, mm_dim, seed), server
    )
    try:
        bound = server.add_insecure_port(f"[::]:{port}")
    except RuntimeError:
        bound = 0
    if bound == 0:
        raise SystemExit(f"error: could not bind gRPC port {port} (already in use?)")
    server.start()
    print(f"SID-MOCK-SERVER READY :{bound}", flush=True)
    server.wait_for_termination()


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--port", type=int, default=50051)
    ap.add_argument("--num-levels", type=int, default=6)
    ap.add_argument("--codebook-size", type=int, default=256)
    ap.add_argument("--mm-dim", type=int, default=1024)
    ap.add_argument("--seed", type=int, default=0)
    args = ap.parse_args()
    serve(args.port, args.num_levels, args.codebook_size, args.mm_dim, args.seed)
    return 0


if __name__ == "__main__":
    sys.exit(main())
