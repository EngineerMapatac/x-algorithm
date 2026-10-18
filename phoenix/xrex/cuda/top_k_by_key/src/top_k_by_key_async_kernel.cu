#include <cuda_bf16.h>
#include <cuda_runtime.h>

#include <algorithm>
#include <cstdint>
#include <cub/device/device_merge_sort.cuh>

#include "top_k_by_key_async_kernel.hpp"
#include "xrex/cuda/xla_utils/cuda_error_utils.hpp"

namespace {

struct Bf16Greater {
  __device__ bool operator()(const nv_bfloat16& a, const nv_bfloat16& b) const { return a > b; }
};

__global__ void strided_gather_kernel(
    const nv_bfloat16* __restrict__ row, int64_t stride, int64_t m, nv_bfloat16* __restrict__ out
) {
  for (int64_t i = blockIdx.x * blockDim.x + threadIdx.x; i < m; i += gridDim.x * blockDim.x) {
    out[i] = row[i * stride];
  }
}

__global__ void fill_neg_inf_kernel(nv_bfloat16* __restrict__ buf, int64_t count) {
  const nv_bfloat16 ninf = __float2bfloat16(-INFINITY);
  for (int64_t i = blockIdx.x * blockDim.x + threadIdx.x; i < count; i += gridDim.x * blockDim.x) {
    buf[i] = ninf;
  }
}

__global__ void bounded_select_kernel(
    const nv_bfloat16* __restrict__ keys,
    int64_t n,
    int64_t cap,
    const nv_bfloat16* __restrict__ pivots,
    nv_bfloat16* __restrict__ surv_keys,
    int32_t* __restrict__ surv_vals,
    int32_t* __restrict__ counts
) {
  const int row = blockIdx.y;
  const nv_bfloat16 pivot = pivots[row];
  const nv_bfloat16* krow = keys + static_cast<int64_t>(row) * n;
  nv_bfloat16* skrow = surv_keys + static_cast<int64_t>(row) * cap;
  int32_t* svrow = surv_vals + static_cast<int64_t>(row) * cap;
  for (int64_t i = blockIdx.x * blockDim.x + threadIdx.x; i < n; i += gridDim.x * blockDim.x) {
    const nv_bfloat16 kv = krow[i];
    if (kv >= pivot) {
      const int pos = atomicAdd(&counts[row], 1);
      if (pos < cap) {
        skrow[pos] = kv;
        svrow[pos] = static_cast<int32_t>(i);
      }
    }
  }
}

void run_async(
    cudaStream_t stream,
    ffi::ScratchAllocator& scratch_allocator,
    const nv_bfloat16* keys_ptr,
    int64_t num_rows,
    int64_t n,
    int64_t k,
    nv_bfloat16* out_keys,
    int32_t* out_vals
) {
  constexpr double kSampleFrac = 0.01;
  constexpr double kSafetyFactor = 4.0;
  constexpr int64_t kSortCapFactor = 64;
  const int64_t target_survivors =
      std::min<int64_t>(std::max<int64_t>(static_cast<int64_t>(kSafetyFactor * k), 1), n);
  const int64_t sample_target =
      std::max<int64_t>(static_cast<int64_t>(kSampleFrac * n), std::min<int64_t>(8 * k, n));
  const int64_t sample_stride = std::max<int64_t>(n / std::max<int64_t>(sample_target, 1), 1);
  const int64_t sample_size = std::max<int64_t>(n / sample_stride, 1);
  const int64_t order_stat_idx =
      std::min<int64_t>(std::max<int64_t>(target_survivors * sample_size / n, 1), sample_size - 1);
  const int64_t cap = std::min<int64_t>(kSortCapFactor * k, n);

  auto alloc = [&](size_t bytes) -> void* { return scratch_allocator.Allocate(bytes).value(); };
  nv_bfloat16* sample_buf = static_cast<nv_bfloat16*>(alloc(sample_size * sizeof(nv_bfloat16)));
  nv_bfloat16* pivots = static_cast<nv_bfloat16*>(alloc(num_rows * sizeof(nv_bfloat16)));
  nv_bfloat16* surv_keys = static_cast<nv_bfloat16*>(alloc(num_rows * cap * sizeof(nv_bfloat16)));
  int32_t* surv_vals = static_cast<int32_t*>(alloc(num_rows * cap * sizeof(int32_t)));
  int32_t* counts = static_cast<int32_t*>(alloc(num_rows * sizeof(int32_t)));

  constexpr int kThreads = 256;
  const Bf16Greater greater_op;

  for (int64_t r = 0; r < num_rows; ++r) {
    const nv_bfloat16* row_ptr = keys_ptr + r * n;
    const int gblocks =
        static_cast<int>(std::min<int64_t>((sample_size + kThreads - 1) / kThreads, 1024));
    strided_gather_kernel<<<gblocks, kThreads, 0, stream>>>(
        row_ptr, sample_stride, sample_size, sample_buf
    );
    size_t tmp_bytes = 0;
    cub::DeviceMergeSort::SortKeys(nullptr, tmp_bytes, sample_buf, sample_size, greater_op, stream);
    void* d_tmp = alloc(tmp_bytes);
    cub::DeviceMergeSort::SortKeys(d_tmp, tmp_bytes, sample_buf, sample_size, greater_op, stream);
    cudaMemcpyAsync(
        pivots + r,
        sample_buf + order_stat_idx,
        sizeof(nv_bfloat16),
        cudaMemcpyDeviceToDevice,
        stream
    );
  }

  cudaMemsetAsync(counts, 0, num_rows * sizeof(int32_t), stream);
  cudaMemsetAsync(surv_vals, 0, num_rows * cap * sizeof(int32_t), stream);
  {
    const int64_t total = num_rows * cap;
    const int fblocks =
        static_cast<int>(std::min<int64_t>((total + kThreads - 1) / kThreads, 4096));
    fill_neg_inf_kernel<<<fblocks, kThreads, 0, stream>>>(surv_keys, total);
  }
  {
    const int xblocks = static_cast<int>(std::min<int64_t>((n + kThreads - 1) / kThreads, 2048));
    const dim3 grid(static_cast<unsigned>(xblocks), static_cast<unsigned>(num_rows));
    bounded_select_kernel<<<grid, kThreads, 0, stream>>>(
        keys_ptr, n, cap, pivots, surv_keys, surv_vals, counts
    );
  }
  for (int64_t r = 0; r < num_rows; ++r) {
    nv_bfloat16* sk = surv_keys + r * cap;
    int32_t* sv = surv_vals + r * cap;
    size_t tmp_bytes = 0;
    cub::DeviceMergeSort::SortPairs(nullptr, tmp_bytes, sk, sv, cap, greater_op, stream);
    void* d_tmp = alloc(tmp_bytes);
    cub::DeviceMergeSort::SortPairs(d_tmp, tmp_bytes, sk, sv, cap, greater_op, stream);
    cudaMemcpyAsync(
        out_keys + r * k, sk, k * sizeof(nv_bfloat16), cudaMemcpyDeviceToDevice, stream
    );
    cudaMemcpyAsync(out_vals + r * k, sv, k * sizeof(int32_t), cudaMemcpyDeviceToDevice, stream);
  }
}

}

ffi::Error top_k_by_key_bf16_async(
    cudaStream_t stream,
    ffi::ScratchAllocator scratch_allocator,
    ffi::Buffer<ffi::DataType::BF16> keys,
    int64_t k,
    ffi::Result<ffi::Buffer<ffi::DataType::BF16>> top_k_keys,
    ffi::Result<ffi::Buffer<ffi::DataType::S32>> top_k_values
) {
  auto dims = keys.dimensions();
  const int64_t num_rows = (dims.size() == 1) ? 1 : dims[0];
  const int64_t n = (dims.size() == 1) ? dims[0] : dims[1];
  if (k <= 0 || k > n) {
    return ffi::Error::InvalidArgument("k must be positive and <= n");
  }

  const nv_bfloat16* keys_ptr = reinterpret_cast<const nv_bfloat16*>(keys.typed_data());
  nv_bfloat16* out_keys = reinterpret_cast<nv_bfloat16*>(top_k_keys->typed_data());
  int32_t* out_vals = reinterpret_cast<int32_t*>(top_k_values->typed_data());

  run_async(stream, scratch_allocator, keys_ptr, num_rows, n, k, out_keys, out_vals);

  XAI_RETURN_IF_CUDA_ERROR(cudaGetLastError());
  return ffi::Error::Success();
}
