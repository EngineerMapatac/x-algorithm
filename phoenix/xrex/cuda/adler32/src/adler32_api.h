#pragma once

#include <cuda_runtime.h>

#include "xla/ffi/api/c_api.h"
#include "xla/ffi/api/ffi.h"

namespace ffi = xla::ffi;

ffi::Error XaiAdler32Host(
    cudaStream_t stream,
    ffi::AnyBuffer input,
    ffi::ResultBufferR0<ffi::U32> sum1,
    ffi::ResultBufferR0<ffi::U32> sum2
);

ffi::Error XaiGlobalAdler32ShardHost(
    cudaStream_t stream,
    ffi::AnyBuffer input,
    ffi::Buffer<ffi::U32> global_shape,
    ffi::Buffer<ffi::U32> shard_shape,
    ffi::Buffer<ffi::U32> shard_offsets,
    ffi::ResultBufferR1<ffi::U32> sums
);

ffi::Error XaiGlobalAdler32ShardFastHost(
    cudaStream_t stream,
    ffi::AnyBuffer input,
    ffi::Buffer<ffi::U32> scales,
    int divider_log2,
    ffi::ResultBufferR1<ffi::U32> sums
);
