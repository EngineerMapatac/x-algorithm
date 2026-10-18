#pragma once

#include <cuda_runtime.h>

#include "xla/ffi/api/ffi.h"

namespace ffi = xla::ffi;

ffi::Error unique_impl_int32(
    cudaStream_t stream,
    ffi::ScratchAllocator scratch_allocator,
    ffi::BufferR1<ffi::DataType::S32> arr,
    const bool return_inverse,
    const int64_t size,
    const int64_t fill_value,
    ffi::Result<ffi::BufferR1<ffi::DataType::S32>> unique_vals,
    ffi::Result<ffi::BufferR1<ffi::DataType::S32>> unique_inverse,
    ffi::Result<ffi::BufferR1<ffi::DataType::S32>> d_keys,
    ffi::Result<ffi::BufferR1<ffi::DataType::S32>> d_indices,
    ffi::Result<ffi::BufferR1<ffi::DataType::S32>> d_temp_buffer
);
