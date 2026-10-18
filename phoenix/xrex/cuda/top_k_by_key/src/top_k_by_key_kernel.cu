#include <cuda_bf16.h>
#include <cuda_runtime.h>
#include <thrust/copy.h>
#include <thrust/count.h>
#include <thrust/device_vector.h>
#include <thrust/execution_policy.h>
#include <thrust/extrema.h>
#include <thrust/functional.h>
#include <thrust/host_vector.h>
#include <thrust/iterator/counting_iterator.h>
#include <thrust/iterator/zip_iterator.h>
#include <thrust/sequence.h>
#include <thrust/sort.h>
#include <thrust/tuple.h>

#include <algorithm>
#include <stdexcept>
#include <vector>

#include "top_k_by_key_kernel.hpp"
#include "xrex/cuda/xla_utils/cuda_error_utils.hpp"
#include "xrex/cuda/xla_utils/thrust_allocator.hpp"

using xrex_cuda::ThrustAllocator;

void top_k_by_key_bf16_1d(
    cudaStream_t stream,
    ffi::ScratchAllocator& scratch_allocator,
    const nv_bfloat16* keys_ptr,
    int64_t n,
    int64_t k,
    double heuristic_pivot_ratio,
    nv_bfloat16 min_key,
    nv_bfloat16 max_key,
    nv_bfloat16* tmp_keys_ptr,
    int32_t* tmp_values_ptr,
    nv_bfloat16* out_keys_ptr,
    int32_t* out_values_ptr
) {
  ThrustAllocator thrust_allocator(&scratch_allocator);
  auto policy = thrust::cuda::par(thrust_allocator).on(stream);

  if (min_key == max_key) {
    thrust::fill(policy, out_keys_ptr, out_keys_ptr + k, min_key);
    thrust::sequence(policy, out_values_ptr, out_values_ptr + k);
    return;
  }

  nv_bfloat16 pivot =
      static_cast<nv_bfloat16>(static_cast<double>(max_key) * heuristic_pivot_ratio);

  auto out_zip = thrust::make_zip_iterator(thrust::make_tuple(tmp_keys_ptr, tmp_values_ptr));
  auto in_zip = thrust::make_zip_iterator(
      thrust::make_tuple(keys_ptr, thrust::counting_iterator<int32_t>(0))
  );
  auto end = thrust::copy_if(
      policy, in_zip, in_zip + n, out_zip, [=] __device__(thrust::tuple<nv_bfloat16, int32_t> tup) {
        return thrust::get<0>(tup) >= pivot;
      }
  );
  int64_t count = end - out_zip;

  if (count >= k) {
    thrust::sort_by_key(
        policy, tmp_keys_ptr, tmp_keys_ptr + count, tmp_values_ptr, thrust::greater<nv_bfloat16>{}
    );
    thrust::copy(policy, tmp_keys_ptr, tmp_keys_ptr + k, out_keys_ptr);
    thrust::copy(policy, tmp_values_ptr, tmp_values_ptr + k, out_values_ptr);
  } else {
    thrust::copy(policy, keys_ptr, keys_ptr + n, tmp_keys_ptr);
    thrust::sequence(policy, tmp_values_ptr, tmp_values_ptr + n);
    thrust::sort_by_key(
        policy, tmp_keys_ptr, tmp_keys_ptr + n, tmp_values_ptr, thrust::greater<nv_bfloat16>{}
    );
    thrust::copy(policy, tmp_keys_ptr, tmp_keys_ptr + k, out_keys_ptr);
    thrust::copy(policy, tmp_values_ptr, tmp_values_ptr + k, out_values_ptr);
  }
}

ffi::Error top_k_by_key_bf16(
    cudaStream_t stream,
    ffi::ScratchAllocator scratch_allocator,
    ffi::Buffer<ffi::DataType::BF16> keys,
    int64_t k,
    double heuristic_pivot_ratio,
    ffi::Result<ffi::Buffer<ffi::DataType::BF16>> top_k_keys,
    ffi::Result<ffi::Buffer<ffi::DataType::S32>> top_k_values
) {
  auto dims = keys.dimensions();
  int64_t num_batches = (dims.size() == 1) ? 1 : dims[0];
  int64_t n = (dims.size() == 1) ? dims[0] : dims[1];
  if (k <= 0 || k > n) {
    return ffi::Error::InvalidArgument("k must be positive and <= n");
  }

  ThrustAllocator thrust_allocator(&scratch_allocator);
  auto policy = thrust::cuda::par(thrust_allocator).on(stream);

  nv_bfloat16* out_keys_ptr = reinterpret_cast<nv_bfloat16*>(top_k_keys->typed_data());
  int32_t* out_values_ptr = reinterpret_cast<int32_t*>(top_k_values->typed_data());
  const nv_bfloat16* keys_ptr = reinterpret_cast<const nv_bfloat16*>(keys.typed_data());

  std::vector<nv_bfloat16> min_keys(num_batches);
  std::vector<nv_bfloat16> max_keys(num_batches);
  for (int64_t batch_idx = 0; batch_idx < num_batches; ++batch_idx) {
    auto minmax =
        thrust::minmax_element(policy, keys_ptr + batch_idx * n, keys_ptr + batch_idx * n + n);
    cudaMemcpyAsync(
        &min_keys[batch_idx],
        thrust::raw_pointer_cast(minmax.first),
        sizeof(nv_bfloat16),
        cudaMemcpyDeviceToHost,
        stream
    );
    cudaMemcpyAsync(
        &max_keys[batch_idx],
        thrust::raw_pointer_cast(minmax.second),
        sizeof(nv_bfloat16),
        cudaMemcpyDeviceToHost,
        stream
    );
  }
  cudaStreamSynchronize(stream);

  nv_bfloat16* tmp_keys_ptr =
      static_cast<nv_bfloat16*>(scratch_allocator.Allocate(n * sizeof(nv_bfloat16)).value());
  int32_t* tmp_values_ptr =
      static_cast<int32_t*>(scratch_allocator.Allocate(n * sizeof(int32_t)).value());

  for (int64_t batch_idx = 0; batch_idx < num_batches; ++batch_idx) {
    top_k_by_key_bf16_1d(
        stream,
        scratch_allocator,
        keys_ptr,
        n,
        k,
        heuristic_pivot_ratio,
        min_keys[batch_idx],
        max_keys[batch_idx],
        tmp_keys_ptr,
        tmp_values_ptr,
        out_keys_ptr,
        out_values_ptr
    );
    keys_ptr += n;
    out_keys_ptr += k;
    out_values_ptr += k;
  }

  XAI_RETURN_IF_CUDA_ERROR(cudaGetLastError());
  return ffi::Error::Success();
}
