#pragma once

#include <cuda_runtime.h>

namespace xrex_mol {

template <int KL, int H>
struct alignas(16) Gate {
  float half_w1[H][KL];
  float w2[H][KL];
  float half_b1[H];
  float b2[KL];
};

template <int KL, int H>
__device__ __forceinline__ void load_gate(
    Gate<KL, H>& g, const float* w1, const float* b1, const float* w2, const float* b2
) {
  for (int t = threadIdx.x; t < KL * H; t += blockDim.x) {
    const int h = t / KL, c = t % KL;
    g.half_w1[h][c] = 0.5f * w1[c * H + h];
    g.w2[h][c] = w2[h * KL + c];
  }
  for (int t = threadIdx.x; t < H; t += blockDim.x) g.half_b1[t] = 0.5f * b1[t];
  for (int t = threadIdx.x; t < KL; t += blockDim.x) g.b2[t] = b2[t];
}

__device__ __forceinline__ void load_shared_x4(const float* src, float* dst) {
  const unsigned addr = static_cast<unsigned>(__cvta_generic_to_shared(src));
  asm volatile("ld.shared.v4.f32 {%0,%1,%2,%3}, [%4];"
               : "=f"(dst[0]), "=f"(dst[1]), "=f"(dst[2]), "=f"(dst[3])
               : "r"(addr));
}

__device__ __forceinline__ float load_shared(const float* src) {
  float v;
  const unsigned addr = static_cast<unsigned>(__cvta_generic_to_shared(src));
  asm volatile("ld.shared.f32 %0, [%1];" : "=f"(v) : "r"(addr));
  return v;
}

__device__ __forceinline__ float tanh_approx(float x) {
  float y;
  asm("tanh.approx.f32 %0, %1;" : "=f"(y) : "f"(x));
  return y;
}

template <int KL, int H, int P>
__device__ __forceinline__ void gate_scores(
    const Gate<KL, H>& g, const float (&cos)[P][KL], float (&logit)[P][KL], float (&out)[P]
) {
  static_assert(KL % 4 == 0, "KL must be a multiple of 4");
#pragma unroll
  for (int h = 0; h < H; ++h) {
    float half_w1[KL], w2[KL];
#pragma unroll
    for (int c = 0; c < KL; c += 4) {
      load_shared_x4(&g.half_w1[h][c], &half_w1[c]);
      load_shared_x4(&g.w2[h][c], &w2[c]);
    }
    const float half_b1 = load_shared(&g.half_b1[h]);
#pragma unroll
    for (int p = 0; p < P; ++p) {
      float half_pre = half_b1;
#pragma unroll
      for (int c = 0; c < KL; ++c) half_pre = fmaf(half_w1[c], cos[p][c], half_pre);
      const float hidden = fmaf(half_pre, tanh_approx(half_pre), half_pre);
#pragma unroll
      for (int c = 0; c < KL; ++c) logit[p][c] = fmaf(w2[c], hidden, logit[p][c]);
    }
  }
#pragma unroll
  for (int p = 0; p < P; ++p) {
    float mx = -3.0e38f;
#pragma unroll
    for (int c = 0; c < KL; ++c) mx = fmaxf(mx, logit[p][c]);
    float den = 0.0f, num = 0.0f;
#pragma unroll
    for (int c = 0; c < KL; ++c) {
      const float e = __expf(logit[p][c] - mx);
      den += e;
      num = fmaf(e, cos[p][c], num);
    }
    out[p] = num * __frcp_rn(den);
  }
}

}
