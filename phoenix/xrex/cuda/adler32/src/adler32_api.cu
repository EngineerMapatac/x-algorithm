
#include <cuda_runtime.h>

#include "adler32_api.h"
#include "adler32_kernel.cuh"
#include "xrex/cuda/xla_utils/cuda_error_utils.hpp"
#include "xla/ffi/api/c_api.h"
#include "xla/ffi/api/ffi.h"

ffi::Error XaiAdler32Host(
    cudaStream_t stream,
    ffi::AnyBuffer input,
    ffi::ResultBufferR0<ffi::U32> sum1,
    ffi::ResultBufferR0<ffi::U32> sum2
) {
  cudaMemsetAsync(sum1->typed_data(), 0, sizeof(uint32_t), stream);
  cudaMemsetAsync(sum2->typed_data(), 0, sizeof(uint32_t), stream);

  const std::size_t size = input.size_bytes();
  uint8_t* input_ptr = static_cast<uint8_t*>(input.untyped_data());
  XAI_CHECK(((uintptr_t)input_ptr & 0xF) == 0, "unaligned memory address");

  const int grid_dim = std::min<int>(1024, (size + BLOCK_DIM - 1) / BLOCK_DIM);
  xai_adler32_kernel<<<grid_dim, BLOCK_DIM, 0, stream>>>(
      reinterpret_cast<const uint8_t*>(input.untyped_data()),
      size,
      sum1->typed_data(),
      sum2->typed_data()
  );

  XAI_CUDA_FFI_CHECK(cudaGetLastError());
  return ffi::Error::Success();
}

ffi::Error XaiGlobalAdler32ShardHost(
    cudaStream_t stream,
    ffi::AnyBuffer input,
    ffi::Buffer<ffi::U32> global_shape,
    ffi::Buffer<ffi::U32> shard_shape,
    ffi::Buffer<ffi::U32> shard_offsets,
    ffi::ResultBufferR1<ffi::U32> sums
) {
  const std::size_t ndim = global_shape.element_count();

  if (ndim == 0) {
    return ffi::Error(XLA_FFI_Error_Code_INTERNAL, std::string("Scalar array not supported"));
  }

  if (ndim != shard_shape.element_count() || ndim != shard_offsets.element_count()) {
    return ffi::Error(XLA_FFI_Error_Code_INTERNAL, std::string("ndim mismatch"));
  }

  if (sums->element_count() != 2) {
    return ffi::Error(XLA_FFI_Error_Code_INTERNAL, "Output needs to be two uint32s");
  }

  cudaMemsetAsync(sums->typed_data(), 0, sizeof(uint32_t[2]), stream);

  const std::size_t size = input.size_bytes();

  const int grid_dim = std::min<int>(1024, (size + BLOCK_DIM - 1) / BLOCK_DIM);

  const uint8_t* d = reinterpret_cast<const uint8_t*>(input.untyped_data());
  const uint32_t* g = global_shape.typed_data();
  const uint32_t* s = shard_shape.typed_data();
  const uint32_t* o = shard_offsets.typed_data();
  uint32_t* out = sums->typed_data();

  switch (ndim) {
    case 1:
      xai_global_adler32_shard_kernel<1><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 2:
      xai_global_adler32_shard_kernel<2><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 3:
      xai_global_adler32_shard_kernel<3><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 4:
      xai_global_adler32_shard_kernel<4><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 5:
      xai_global_adler32_shard_kernel<5><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 6:
      xai_global_adler32_shard_kernel<6><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 7:
      xai_global_adler32_shard_kernel<7><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 8:
      xai_global_adler32_shard_kernel<8><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 9:
      xai_global_adler32_shard_kernel<9><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 10:
      xai_global_adler32_shard_kernel<10><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 11:
      xai_global_adler32_shard_kernel<11><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 12:
      xai_global_adler32_shard_kernel<12><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 13:
      xai_global_adler32_shard_kernel<13><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    case 14:
      xai_global_adler32_shard_kernel<14><<<grid_dim, BLOCK_DIM, 0, stream>>>(d, g, s, o, out);
      break;
    default:
      return ffi::Error(
          XLA_FFI_Error_Code_INTERNAL, std::string("Unsupported dimension: ") + std::to_string(ndim)
      );
  }

  XAI_CUDA_FFI_CHECK(cudaGetLastError());
  return ffi::Error::Success();
}

ffi::Error XaiGlobalAdler32ShardFastHost(
    cudaStream_t stream,
    ffi::AnyBuffer input,
    ffi::Buffer<ffi::U32> scales,
    int divider_log2,
    ffi::ResultBufferR1<ffi::U32> sums
) {
  cudaMemsetAsync(sums->typed_data(), 0, sums->size_bytes(), stream);
  uint8_t* input_ptr = static_cast<uint8_t*>(input.untyped_data());
  size_t nbytes = input.size_bytes();
  uint32_t* scales_ptr = static_cast<uint32_t*>(scales.untyped_data());

  uint32_t* sums_ptr = static_cast<uint32_t*>(sums->untyped_data());

  cudaDeviceProp device_prop;
  XAI_CUDA_CHECK(cudaGetDeviceProperties(&device_prop, 0));

  int num_blocks_per_sm = 0;
  XAI_CUDA_CHECK(cudaOccupancyMaxActiveBlocksPerMultiprocessor(
      &num_blocks_per_sm, xai_adler32_fast_kernel, BLOCK_DIM, 0
  ));

  XAI_CHECK(
      ((uintptr_t)input_ptr & 0xF) == 0, "unaligned memory address"
  );
  xai_adler32_fast_kernel<<<
      device_prop.multiProcessorCount * num_blocks_per_sm,
      BLOCK_DIM,
      0,
      stream>>>(input_ptr, scales_ptr, nbytes, sums_ptr, sums_ptr + 1, divider_log2);

  XAI_CUDA_FFI_CHECK(cudaGetLastError());
  return ffi::Error::Success();
}
