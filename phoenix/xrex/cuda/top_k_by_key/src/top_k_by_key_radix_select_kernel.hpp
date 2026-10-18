#pragma once

#include <cuda_runtime.h>

#include "xla/ffi/api/ffi.h"

namespace ffi = xla::ffi;

ffi::Error top_k_by_key_bf16_radix_select(
    cudaStream_t stream,
    ffi::ScratchAllocator scratch_allocator,
    ffi::Buffer<ffi::DataType::BF16> keys,
    int64_t k,
    ffi::Result<ffi::Buffer<ffi::DataType::BF16>> top_k_keys,
    ffi::Result<ffi::Buffer<ffi::DataType::S32>> top_k_values
);
