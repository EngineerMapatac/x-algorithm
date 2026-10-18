#include <cuda_bf16.h>
#include <cuda_runtime.h>

#include <cstdint>

#include "mol_epilogue_kernel.hpp"
#include "mol_gate.hpp"
#include "xrex/cuda/xla_utils/cuda_error_utils.hpp"

namespace {

constexpr int kMaxUsers = 512;
constexpr int kThreads = 128;

struct Params {
  const __nv_bfloat16* dots;
  const float* norms;
  const float* ubias;
  const float* ibias;
  const float* w1;
  const float* b1;
  const float* w2;
  const float* b2;
  float* out;
  int num_users;
  int num_components;
  long long num_posts;
};

template <int KL>
constexpr int kUsersPerThread = KL <= 4 ? 3 : (KL <= 8 ? 2 : 1);

template <int KL, int H, int U>
__device__ __forceinline__ void score_users(
    const Params& p,
    const xrex_mol::Gate<KL, H>& g,
    const float* s_ubias,
    const float* inv_norm,
    const float* ib,
    int b0,
    long long m
) {
  const long long M = p.num_posts;
  const int B = p.num_users;
  float cos[U][KL];
#pragma unroll
  for (int c = 0; c < KL; ++c) {
    const long long row = static_cast<long long>(c) * B + b0;
#pragma unroll
    for (int u = 0; u < U; ++u) cos[u][c] = __bfloat162float(p.dots[(row + u) * M + m]) * inv_norm[c];
  }
  float logit[U][KL];
#pragma unroll
  for (int u = 0; u < U; ++u)
#pragma unroll
    for (int c = 0; c < KL; ++c) logit[u][c] = g.b2[c] + ib[c] + s_ubias[c * B + b0 + u];
  float out[U];
  xrex_mol::gate_scores<KL, H, U>(g, cos, logit, out);
#pragma unroll
  for (int u = 0; u < U; ++u) p.out[static_cast<long long>(b0 + u) * M + m] = out[u];
}

template <int KL, int H>
__global__ void __launch_bounds__(kThreads) mol_epilogue_kernel(Params p) {
  constexpr int U = kUsersPerThread<KL>;
  __shared__ xrex_mol::Gate<KL, H> g;
  __shared__ float s_ubias[KL * kMaxUsers];
  xrex_mol::load_gate<KL, H>(g, p.w1, p.b1, p.w2, p.b2);
  for (int i = threadIdx.x; i < KL * p.num_users; i += blockDim.x) s_ubias[i] = p.ubias[i];
  __syncthreads();

  const long long m = static_cast<long long>(blockIdx.x) * blockDim.x + threadIdx.x;
  if (m >= p.num_posts)
    return;
  const long long M = p.num_posts;
  const int B = p.num_users;

  float inv_norm[KL];
  float ib[KL];
#pragma unroll
  for (int c = 0; c < KL; ++c) {
    inv_norm[c] = 1.0f / p.norms[static_cast<long long>(c % p.num_components) * M + m];
    ib[c] = p.ibias[static_cast<long long>(c) * M + m];
  }

  int b = 0;
  for (; b + U <= B; b += U) score_users<KL, H, U>(p, g, s_ubias, inv_norm, ib, b, m);
  for (; b < B; ++b) score_users<KL, H, 1>(p, g, s_ubias, inv_norm, ib, b, m);
}

}

ffi::Error mol_epilogue(
    cudaStream_t stream,
    ffi::Buffer<ffi::DataType::BF16> dots,
    ffi::Buffer<ffi::DataType::F32> norms,
    ffi::Buffer<ffi::DataType::F32> ubias,
    ffi::Buffer<ffi::DataType::F32> ibias,
    ffi::Buffer<ffi::DataType::F32> w1,
    ffi::Buffer<ffi::DataType::F32> b1,
    ffi::Buffer<ffi::DataType::F32> w2,
    ffi::Buffer<ffi::DataType::F32> b2,
    ffi::Result<ffi::Buffer<ffi::DataType::F32>> out,
    int64_t num_users,
    int64_t num_components
) {
  const auto dots_dims = dots.dimensions();
  const auto w1_dims = w1.dimensions();
  if (dots_dims.size() != 2 || w1_dims.size() != 2) {
    return ffi::Error::InvalidArgument("dots must be [KL*B, M] and w1 [KL, H]");
  }
  const int64_t kl = w1_dims[0];
  const int64_t hidden = w1_dims[1];
  const int64_t rows = dots_dims[0];
  const int64_t M = dots_dims[1];
  if (rows != kl * num_users)
    return ffi::Error::InvalidArgument("dots rows != KL * num_users");
  if (num_users > kMaxUsers)
    return ffi::Error::InvalidArgument("too many users for the kernel");
  if (norms.dimensions().size() != 2 || norms.dimensions()[0] != num_components ||
      norms.dimensions()[1] != M) {
    return ffi::Error::InvalidArgument("norms must be [L, M]");
  }
  if (ibias.dimensions().size() != 2 || ibias.dimensions()[0] != kl || ibias.dimensions()[1] != M) {
    return ffi::Error::InvalidArgument("ibias must be [KL, M]");
  }
  if (ubias.dimensions().size() != 2 || ubias.dimensions()[0] != kl ||
      ubias.dimensions()[1] != num_users) {
    return ffi::Error::InvalidArgument("ubias must be [KL, B]");
  }
  Params p{
      reinterpret_cast<const __nv_bfloat16*>(dots.typed_data()),
      norms.typed_data(),
      ubias.typed_data(),
      ibias.typed_data(),
      w1.typed_data(),
      b1.typed_data(),
      w2.typed_data(),
      b2.typed_data(),
      out->typed_data(),
      static_cast<int>(num_users),
      static_cast<int>(num_components),
      M
  };
  const unsigned blocks = static_cast<unsigned>((M + kThreads - 1) / kThreads);
  if (kl == 4 && hidden == 32) {
    mol_epilogue_kernel<4, 32><<<blocks, kThreads, 0, stream>>>(p);
  } else if (kl == 8 && hidden == 16) {
    mol_epilogue_kernel<8, 16><<<blocks, kThreads, 0, stream>>>(p);
  } else if (kl == 8 && hidden == 32) {
    mol_epilogue_kernel<8, 32><<<blocks, kThreads, 0, stream>>>(p);
  } else if (kl == 16 && hidden == 16) {
    mol_epilogue_kernel<16, 16><<<blocks, kThreads, 0, stream>>>(p);
  } else {
    return ffi::Error::InvalidArgument(
        "unsupported (KL, H); compiled: (4,32), (8,16), (8,32), (16,16)"
    );
  }
  XAI_RETURN_IF_CUDA_ERROR(cudaGetLastError());
  return ffi::Error::Success();
}
