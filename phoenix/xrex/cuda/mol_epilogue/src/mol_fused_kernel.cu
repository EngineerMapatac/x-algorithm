#include <cuda_bf16.h>
#include <cuda_runtime.h>

#include <algorithm>
#include <cstdint>

#include "mol_epilogue_kernel.hpp"
#include "mol_gate.hpp"
#include "xrex/cuda/xla_utils/cuda_error_utils.hpp"

namespace {

constexpr int KL = 4;
constexpr int H = 32;
constexpr int BM = 128;
constexpr int BK = 64;
constexpr int STAGES = 5;
constexpr int PRODUCER_WARPS = 8;
constexpr int CONSUMER_WARPS = 8;
constexpr int PRODUCERS = 32 * PRODUCER_WARPS;
constexpr int CONSUMERS = 32 * CONSUMER_WARPS;
constexpr int THREADS = PRODUCERS + CONSUMERS;
constexpr int BAR_PRODUCERS = 1;
constexpr int BAR_FULL = 2;
constexpr int BAR_EMPTY = 4;

struct Params {
  const int8_t* posts;
  const float* post_scales;
  const int8_t* queries;
  const float* query_scales;
  const float* norms;
  const float* ubias;
  const float* ibias;
  const float* w1;
  const float* b1;
  const float* w2;
  const float* b2;
  __nv_bfloat16* out;
  int num_users;
  long long num_posts;
  int dim;
  long long row_stride;
  long long col_offset;
};

template <int R>
constexpr size_t dynamic_smem_bytes() {
  return static_cast<size_t>(STAGES) * (BM + R) * BK + 2 * static_cast<size_t>(R / KL) * BM * sizeof(uint2);
}

__device__ __forceinline__ unsigned smem_addr(const void* p) {
  return static_cast<unsigned>(__cvta_generic_to_shared(p));
}

__device__ __forceinline__ unsigned swizzled(int row, int chunk) {
  return row * BK + ((chunk ^ ((row >> 1) & 3)) << 4);
}

__device__ __forceinline__ void cp_async16(unsigned dst, const void* src, bool valid) {
  asm volatile("cp.async.cg.shared.global [%0], [%1], 16, %2;" ::"r"(dst), "l"(src), "r"(valid ? 16 : 0));
}

__device__ __forceinline__ void bar_sync(int id, int n) {
  asm volatile("bar.sync %0, %1;" ::"r"(id), "r"(n) : "memory");
}

__device__ __forceinline__ void bar_arrive(int id, int n) {
  asm volatile("bar.arrive %0, %1;" ::"r"(id), "r"(n) : "memory");
}

__device__ __forceinline__ void mma_s8(int* d, const unsigned* a, unsigned b0, unsigned b1) {
  asm volatile(
      "mma.sync.aligned.m16n8k32.row.col.s32.s8.s8.s32 {%0,%1,%2,%3}, {%4,%5,%6,%7}, {%8,%9}, {%0,%1,%2,%3};"
      : "+r"(d[0]), "+r"(d[1]), "+r"(d[2]), "+r"(d[3])
      : "r"(a[0]), "r"(a[1]), "r"(a[2]), "r"(a[3]), "r"(b0), "r"(b1)
  );
}

__device__ __forceinline__ void ldmatrix_x4(unsigned* r, unsigned addr) {
  asm volatile("ldmatrix.sync.aligned.m8n8.x4.shared.b16 {%0,%1,%2,%3}, [%4];"
               : "=r"(r[0]), "=r"(r[1]), "=r"(r[2]), "=r"(r[3])
               : "r"(addr));
}

template <int R>
__device__ __forceinline__ void produce(
    const Params& p, int8_t* sa, int8_t* sb, uint2* dots, const float* s_qs, long long num_tiles
) {
  constexpr int NT = R / 16;
  constexpr int USERS = R / KL;
  const int warp = threadIdx.x >> 5, lane = threadIdx.x & 31;
  const int wm = warp % 4, wn = warp / 4;
  const int KT = p.dim / BK;
  const int D = p.dim;
  const long long M = p.num_posts;
  const int gid = lane >> 2, tig = lane & 3;
  const bool odd = tig & 1;
  const int mi = lane >> 3;
  const unsigned sa0 = smem_addr(sa), sb0 = smem_addr(sb);
  unsigned off_a[2][2], off_b[2][NT / 2];
#pragma unroll
  for (int ks = 0; ks < 2; ++ks) {
#pragma unroll
    for (int i = 0; i < 2; ++i)
      off_a[ks][i] = swizzled(wm * 32 + i * 16 + (mi & 1) * 8 + (lane & 7), 2 * ks + (mi >> 1));
#pragma unroll
    for (int j = 0; j < NT; j += 2)
      off_b[ks][j / 2] = swizzled(wn * (R / 2) + j * 8 + (mi >> 1) * 8 + (lane & 7), 2 * ks + (mi & 1));
  }
  const int lrow = threadIdx.x >> 2, lchunk = threadIdx.x & 3;
  const unsigned dst0 = swizzled(lrow, lchunk), dst1 = swizzled(lrow + 64, lchunk);
  const bool has_q0 = threadIdx.x < R * 4, has_q1 = threadIdx.x + PRODUCERS < R * 4;
  const int8_t* q0 = p.queries + static_cast<long long>(lrow) * D + lchunk * 16;
  const int8_t* q1 = q0 + 64LL * D;

  int it = 0;
  for (long long tile = blockIdx.x; tile < num_tiles; tile += gridDim.x, ++it) {
    const long long m0 = tile * BM;
    const bool ok0 = m0 + lrow < M, ok1 = m0 + lrow + 64 < M;
    const int8_t* p0 = p.posts + (ok0 ? m0 + lrow : 0) * p.row_stride + p.col_offset + lchunk * 16;
    const int8_t* p1 = p.posts + (ok1 ? m0 + lrow + 64 : 0) * p.row_stride + p.col_offset + lchunk * 16;
    auto issue = [&](int stage, int kt) {
      const unsigned a = sa0 + stage * BM * BK, b = sb0 + stage * R * BK;
      const int k0 = kt * BK;
      cp_async16(a + dst0, p0 + k0, ok0);
      cp_async16(a + dst1, p1 + k0, ok1);
      if (has_q0) cp_async16(b + dst0, q0 + k0, true);
      if (has_q1) cp_async16(b + dst1, q1 + k0, true);
    };
#pragma unroll
    for (int st = 0; st < STAGES - 1; ++st) {
      if (st < KT) issue(st, st);
      asm volatile("cp.async.commit_group;");
    }
    int acc[2][NT][4];
#pragma unroll
    for (int i = 0; i < 2; ++i)
#pragma unroll
      for (int j = 0; j < NT; ++j)
#pragma unroll
        for (int e = 0; e < 4; ++e) acc[i][j][e] = 0;
    int stage = 0;
    for (int kt = 0; kt < KT; ++kt) {
      asm volatile("cp.async.wait_group %0;" ::"n"(STAGES - 2));
      bar_sync(BAR_PRODUCERS, PRODUCERS);
      int next = stage + STAGES - 1;
      if (next >= STAGES) next -= STAGES;
      if (kt + STAGES - 1 < KT) issue(next, kt + STAGES - 1);
      asm volatile("cp.async.commit_group;");
      const unsigned a = sa0 + stage * BM * BK, b = sb0 + stage * R * BK;
#pragma unroll
      for (int ks = 0; ks < 2; ++ks) {
        unsigned af[2][4];
#pragma unroll
        for (int i = 0; i < 2; ++i) ldmatrix_x4(af[i], a + off_a[ks][i]);
#pragma unroll
        for (int j = 0; j < NT; j += 2) {
          unsigned bf[4];
          ldmatrix_x4(bf, b + off_b[ks][j / 2]);
#pragma unroll
          for (int i = 0; i < 2; ++i) {
            mma_s8(acc[i][j], af[i], bf[0], bf[1]);
            mma_s8(acc[i][j + 1], af[i], bf[2], bf[3]);
          }
        }
      }
      if (++stage == STAGES) stage = 0;
    }
    asm volatile("cp.async.wait_group 0;");
    bar_sync(BAR_PRODUCERS, PRODUCERS);

    const int buf = it & 1;
    if (it >= 2) bar_sync(BAR_EMPTY + buf, THREADS);
    uint2* tile_dots = dots + buf * USERS * BM;
#pragma unroll
    for (int i = 0; i < 2; ++i) {
      const int row = wm * 32 + i * 16 + (odd ? gid + 8 : gid);
      const float ps = p.post_scales[m0 + row < M ? m0 + row : 0];
#pragma unroll
      for (int j = 0; j < NT; ++j) {
        const int* a4 = acc[i][j];
        const int r0 = __shfl_xor_sync(0xffffffffu, odd ? a4[0] : a4[2], 1);
        const int r1 = __shfl_xor_sync(0xffffffffu, odd ? a4[1] : a4[3], 1);
        const int comp[KL] = {odd ? r0 : a4[0], odd ? r1 : a4[1], odd ? a4[2] : r0, odd ? a4[3] : r1};
        const int user = (wn * NT + j) * 2 + (tig >> 1);
        __nv_bfloat162 lo, hi;
        lo.x = __float2bfloat16_rn((static_cast<float>(comp[0]) * s_qs[user * KL + 0]) * ps);
        lo.y = __float2bfloat16_rn((static_cast<float>(comp[1]) * s_qs[user * KL + 1]) * ps);
        hi.x = __float2bfloat16_rn((static_cast<float>(comp[2]) * s_qs[user * KL + 2]) * ps);
        hi.y = __float2bfloat16_rn((static_cast<float>(comp[3]) * s_qs[user * KL + 3]) * ps);
        tile_dots[user * BM + row] = make_uint2(reinterpret_cast<unsigned&>(lo), reinterpret_cast<unsigned&>(hi));
      }
    }
    bar_arrive(BAR_FULL + buf, THREADS);
  }
}

template <int R>
__device__ __forceinline__ void consume(
    const Params& p,
    const xrex_mol::Gate<KL, H>& g,
    const uint2* dots,
    const float* s_ub,
    long long num_tiles
) {
  constexpr int USERS = R / KL;
  constexpr int USER_GROUPS = CONSUMERS / BM;
  constexpr int PER_THREAD = USERS / USER_GROUPS;
#if __CUDA_ARCH__ >= 900
  constexpr int P = PER_THREAD % 8 == 0 ? 8 : 4;
#else
  constexpr int P = 4;
#endif
  static_assert(PER_THREAD % P == 0, "users per consumer thread must be a multiple of P");
  const int ctid = threadIdx.x - PRODUCERS;
  const int row = ctid % BM;
  const int group = ctid / BM;
  const int B = p.num_users;
  const long long M = p.num_posts;
  long long tiles_here = 0;
  for (long long tile = blockIdx.x; tile < num_tiles; tile += gridDim.x) ++tiles_here;

  int it = 0;
  for (long long tile = blockIdx.x; tile < num_tiles; tile += gridDim.x, ++it) {
    const long long m = tile * BM + row;
    const bool valid = m < M;
    const long long mc = valid ? m : 0;
    const float inv_norm = 1.0f / p.norms[mc];
    float ib[KL];
#pragma unroll
    for (int c = 0; c < KL; ++c) ib[c] = p.ibias[c * M + mc];
    const int buf = it & 1;
    bar_sync(BAR_FULL + buf, THREADS);
    const uint2* tile_dots = dots + buf * USERS * BM;
#pragma unroll
    for (int u0 = 0; u0 < PER_THREAD; u0 += P) {
      float cos[P][KL], logit[P][KL], out[P];
#pragma unroll
      for (int q = 0; q < P; ++q) {
        const int user = (u0 + q) * USER_GROUPS + group;
        const uint2 v = tile_dots[user * BM + row];
        const __nv_bfloat162 lo = reinterpret_cast<const __nv_bfloat162&>(v.x);
        const __nv_bfloat162 hi = reinterpret_cast<const __nv_bfloat162&>(v.y);
        cos[q][0] = __bfloat162float(lo.x) * inv_norm;
        cos[q][1] = __bfloat162float(lo.y) * inv_norm;
        cos[q][2] = __bfloat162float(hi.x) * inv_norm;
        cos[q][3] = __bfloat162float(hi.y) * inv_norm;
#pragma unroll
        for (int c = 0; c < KL; ++c) logit[q][c] = g.b2[c] + ib[c] + s_ub[user * KL + c];
      }
      xrex_mol::gate_scores<KL, H, P>(g, cos, logit, out);
#pragma unroll
      for (int q = 0; q < P; ++q) {
        const int user = (u0 + q) * USER_GROUPS + group;
        if (valid && user < B) p.out[static_cast<long long>(user) * M + m] = __float2bfloat16_rn(out[q]);
      }
    }
    if (it + 2 < tiles_here) bar_arrive(BAR_EMPTY + buf, THREADS);
  }
}

template <int R>
__global__ void __launch_bounds__(THREADS, 1) mol_fused_int8_kernel(Params p) {
  extern __shared__ __align__(128) int8_t smem[];
  __shared__ xrex_mol::Gate<KL, H> g;
  __shared__ float s_qs[R];
  __shared__ float s_ub[R];
  int8_t* sa = smem;
  int8_t* sb = smem + STAGES * BM * BK;
  uint2* dots = reinterpret_cast<uint2*>(smem + STAGES * (BM + R) * BK);
  const long long num_tiles = (p.num_posts + BM - 1) / BM;

  xrex_mol::load_gate<KL, H>(g, p.w1, p.b1, p.w2, p.b2);
  for (int i = threadIdx.x; i < R; i += THREADS) {
    const int b = i / KL, c = i % KL;
    s_qs[i] = p.query_scales[i];
    s_ub[i] = b < p.num_users ? p.ubias[c * p.num_users + b] : 0.0f;
  }
  __syncthreads();
  if (threadIdx.x < PRODUCERS) {
    produce<R>(p, sa, sb, dots, s_qs, num_tiles);
  } else {
    consume<R>(p, g, dots, s_ub, num_tiles);
  }
}

template <int R>
ffi::Error launch(const Params& p, cudaStream_t stream) {
  constexpr size_t smem = dynamic_smem_bytes<R>();
  XAI_RETURN_IF_CUDA_ERROR(cudaFuncSetAttribute(
      mol_fused_int8_kernel<R>, cudaFuncAttributeMaxDynamicSharedMemorySize, static_cast<int>(smem)
  ));
  int device = 0, sms = 0;
  XAI_RETURN_IF_CUDA_ERROR(cudaGetDevice(&device));
  XAI_RETURN_IF_CUDA_ERROR(cudaDeviceGetAttribute(&sms, cudaDevAttrMultiProcessorCount, device));
  const long long tiles = (p.num_posts + BM - 1) / BM;
  const unsigned blocks = static_cast<unsigned>(std::max<long long>(1, std::min<long long>(tiles, sms)));
  mol_fused_int8_kernel<R><<<blocks, THREADS, smem, stream>>>(p);
  XAI_RETURN_IF_CUDA_ERROR(cudaGetLastError());
  return ffi::Error::Success();
}

}

ffi::Error mol_fused_int8(
    cudaStream_t stream,
    ffi::Buffer<ffi::DataType::S8> posts,
    ffi::Buffer<ffi::DataType::F32> post_scales,
    ffi::Buffer<ffi::DataType::S8> queries,
    ffi::Buffer<ffi::DataType::F32> query_scales,
    ffi::Buffer<ffi::DataType::F32> norms,
    ffi::Buffer<ffi::DataType::F32> ubias,
    ffi::Buffer<ffi::DataType::F32> ibias,
    ffi::Buffer<ffi::DataType::F32> w1,
    ffi::Buffer<ffi::DataType::F32> b1,
    ffi::Buffer<ffi::DataType::F32> w2,
    ffi::Buffer<ffi::DataType::F32> b2,
    ffi::Result<ffi::Buffer<ffi::DataType::BF16>> out,
    int64_t num_users,
    int64_t col_offset
) {
  const auto pd = posts.dimensions();
  const auto qd = queries.dimensions();
  if (pd.size() != 2 || qd.size() != 2)
    return ffi::Error::InvalidArgument("posts must be [M, S] and queries [R, D]");
  const int64_t M = pd[0], S = pd[1], D = qd[1], R = qd[0];
  if (D % BK != 0)
    return ffi::Error::InvalidArgument("D must be a multiple of 64");
  if (S % 16 != 0 || col_offset < 0 || col_offset % 16 != 0 || col_offset + D > S)
    return ffi::Error::InvalidArgument("posts columns [col_offset, col_offset + D) must lie in a row, 16-byte aligned");
  if (w1.dimensions().size() != 2 || w1.dimensions()[0] != KL || w1.dimensions()[1] != H)
    return ffi::Error::InvalidArgument("the fused kernel supports KL=4, H=32 only");
  if (num_users <= 0 || num_users * KL > R)
    return ffi::Error::InvalidArgument("queries must hold KL rows per user");
  if (norms.dimensions().size() != 2 || norms.dimensions()[0] != 1 || norms.dimensions()[1] != M)
    return ffi::Error::InvalidArgument("norms must be [1, M]");
  if (ibias.dimensions().size() != 2 || ibias.dimensions()[0] != KL || ibias.dimensions()[1] != M)
    return ffi::Error::InvalidArgument("ibias must be [KL, M]");
  if (ubias.dimensions().size() != 2 || ubias.dimensions()[0] != KL || ubias.dimensions()[1] != num_users)
    return ffi::Error::InvalidArgument("ubias must be [KL, B]");
  if (post_scales.element_count() != static_cast<size_t>(M) || query_scales.element_count() != static_cast<size_t>(R))
    return ffi::Error::InvalidArgument("post_scales must be [M] and query_scales [R]");
  Params p{
      posts.typed_data(),
      post_scales.typed_data(),
      queries.typed_data(),
      query_scales.typed_data(),
      norms.typed_data(),
      ubias.typed_data(),
      ibias.typed_data(),
      w1.typed_data(),
      b1.typed_data(),
      w2.typed_data(),
      b2.typed_data(),
      reinterpret_cast<__nv_bfloat16*>(out->typed_data()),
      static_cast<int>(num_users),
      M,
      static_cast<int>(D),
      S,
      col_offset
  };
  switch (R) {
    case 32:
      return launch<32>(p, stream);
    case 64:
      return launch<64>(p, stream);
    case 96:
      return launch<96>(p, stream);
    case 128:
      return launch<128>(p, stream);
    default:
      return ffi::Error::InvalidArgument("queries rows must be 32, 64, 96 or 128");
  }
}
