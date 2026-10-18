# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from __future__ import annotations

import ctypes
import logging

import cupy as cp
import jax
import numpy as np

logger = logging.getLogger(__name__)

_MEMCPY_D2H = 2


class PinnedD2HBuffer:
    def __init__(
        self,
        shape: tuple[int, ...],
        dtype: np.dtype,
        num_shards: int,
        num_buffers: int = 3,
    ) -> None:
        if num_buffers < 2:
            raise ValueError(f"num_buffers must be >= 2, got {num_buffers}")

        self._shape = shape
        self._dtype = np.dtype(dtype)
        self._num_shards = num_shards
        self._num_buffers = num_buffers
        self._nbytes = int(np.prod(shape)) * self._dtype.itemsize

        self._pinned_mems = [cp.cuda.alloc_pinned_memory(self._nbytes) for _ in range(num_buffers)]

        ctype = np.ctypeslib.as_ctypes_type(self._dtype)
        self._outs = []
        for pinned_mem in self._pinned_mems:
            ptr = ctypes.cast(pinned_mem.ptr, ctypes.POINTER(ctype))
            self._outs.append(np.ctypeslib.as_array(ptr, shape=shape))

        self._buf_idx = 0

        self._transfers_since_cycle: int | None = None

        logger.info(
            f"PinnedD2HBuffer: allocated {num_buffers * self._nbytes / 1024 / 1024:.1f} MB "
            f"pinned memory ({num_buffers}x{self._nbytes / 1024 / 1024:.1f} MB ring) "
            f"for shape {shape}, dtype {dtype}, {num_shards} shards"
        )

    def transfer(self, arr: jax.Array) -> np.ndarray:
        if arr.shape != self._shape:
            raise ValueError(
                f"Array shape {arr.shape} does not match buffer shape {self._shape}. "
                f"Create a new PinnedD2HBuffer for different shapes."
            )

        if np.dtype(arr.dtype) != self._dtype:
            raise ValueError(
                f"Array dtype {arr.dtype} does not match buffer dtype {self._dtype}. "
                f"Convert array to {self._dtype} before transfer."
            )

        shards = arr.addressable_shards
        assert len(shards) == self._num_shards, (
            f"Expected {self._num_shards} shards, got {len(shards)}"
        )

        if self._transfers_since_cycle is not None:
            self._transfers_since_cycle += 1
            if self._transfers_since_cycle > self._num_buffers:
                raise RuntimeError(
                    f"PinnedD2HBuffer ring overflow: more than {self._num_buffers} transfers "
                    f"in one cycle for shape {self._shape} — would overwrite a live view."
                )

        buf_idx = self._buf_idx
        self._buf_idx = (self._buf_idx + 1) % self._num_buffers

        host_ptr = self._pinned_mems[buf_idx].ptr
        stride = int(np.prod(self._shape[1:])) * self._dtype.itemsize

        for shard in shards:
            cp.cuda.runtime.setDevice(shard.device.id)
            idx = shard.index
            row_start = idx[0].start or 0
            offset_bytes = row_start * stride
            shard_bytes = int(np.prod(shard.data.shape)) * self._dtype.itemsize
            cai = shard.data.__cuda_array_interface__
            dev_ptr = cai["data"][0]
            cp.cuda.runtime.memcpyAsync(
                host_ptr + offset_bytes,
                dev_ptr,
                shard_bytes,
                _MEMCPY_D2H,
                0,
            )

        for shard in shards:
            cp.cuda.runtime.setDevice(shard.device.id)
            cp.cuda.runtime.streamSynchronize(0)

        return self._outs[buf_idx]

    def begin_cycle(self) -> None:
        self._transfers_since_cycle = 0

    def __del__(self) -> None:
        pass
