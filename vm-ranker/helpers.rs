use half::f16;

#[inline]
pub(crate) fn dot_product(a: &[f16], b: &[f16]) -> f64 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            return unsafe { dot_product_avx512(a, b) };
        }
        if is_x86_feature_detected!("avx2")
            && is_x86_feature_detected!("fma")
            && is_x86_feature_detected!("f16c")
        {
            return unsafe { dot_product_avx2_fma_f16c(a, b) };
        }
    }
    dot_product_scalar(a, b)
}

#[inline]
fn dot_product_scalar(a: &[f16], b: &[f16]) -> f64 {
    let len = a.len().min(b.len());
    let mut dot = 0.0f64;
    for i in 0..len {
        dot += (a[i].to_f32() * b[i].to_f32()) as f64;
    }
    dot
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma,f16c")]
unsafe fn dot_product_avx2_fma_f16c(a: &[f16], b: &[f16]) -> f64 {
    use std::arch::x86_64::*;

    unsafe {
        let len = a.len().min(b.len());
        let chunks = len / 32;

        let mut acc0 = _mm256_setzero_ps();
        let mut acc1 = _mm256_setzero_ps();
        let mut acc2 = _mm256_setzero_ps();
        let mut acc3 = _mm256_setzero_ps();

        let a_ptr = a.as_ptr() as *const u16;
        let b_ptr = b.as_ptr() as *const u16;

        for i in 0..chunks {
            let base = i * 32;

            let a0 = _mm256_cvtph_ps(_mm_loadu_si128(a_ptr.add(base) as *const __m128i));
            let b0 = _mm256_cvtph_ps(_mm_loadu_si128(b_ptr.add(base) as *const __m128i));
            acc0 = _mm256_fmadd_ps(a0, b0, acc0);

            let a1 = _mm256_cvtph_ps(_mm_loadu_si128(a_ptr.add(base + 8) as *const __m128i));
            let b1 = _mm256_cvtph_ps(_mm_loadu_si128(b_ptr.add(base + 8) as *const __m128i));
            acc1 = _mm256_fmadd_ps(a1, b1, acc1);

            let a2 = _mm256_cvtph_ps(_mm_loadu_si128(a_ptr.add(base + 16) as *const __m128i));
            let b2 = _mm256_cvtph_ps(_mm_loadu_si128(b_ptr.add(base + 16) as *const __m128i));
            acc2 = _mm256_fmadd_ps(a2, b2, acc2);

            let a3 = _mm256_cvtph_ps(_mm_loadu_si128(a_ptr.add(base + 24) as *const __m128i));
            let b3 = _mm256_cvtph_ps(_mm_loadu_si128(b_ptr.add(base + 24) as *const __m128i));
            acc3 = _mm256_fmadd_ps(a3, b3, acc3);
        }

        acc0 = _mm256_add_ps(acc0, acc1);
        acc2 = _mm256_add_ps(acc2, acc3);
        acc0 = _mm256_add_ps(acc0, acc2);

        let hi = _mm256_extractf128_ps(acc0, 1);
        let lo = _mm256_castps256_ps128(acc0);
        let sum128 = _mm_add_ps(lo, hi);
        let shuf = _mm_movehdup_ps(sum128);
        let sums = _mm_add_ps(sum128, shuf);
        let shuf2 = _mm_movehl_ps(sums, sums);
        let result = _mm_add_ss(sums, shuf2);

        let mut dot = _mm_cvtss_f32(result) as f64;
        let tail_start = chunks * 32;
        for i in tail_start..len {
            let af = f16::from_bits(*a_ptr.add(i)).to_f32();
            let bf = f16::from_bits(*b_ptr.add(i)).to_f32();
            dot += (af * bf) as f64;
        }
        dot
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn dot_product_avx512(a: &[f16], b: &[f16]) -> f64 {
    use std::arch::x86_64::*;

    unsafe {
        let len = a.len().min(b.len());
        let chunks = len / 64;

        let mut acc0 = _mm512_setzero_ps();
        let mut acc1 = _mm512_setzero_ps();
        let mut acc2 = _mm512_setzero_ps();
        let mut acc3 = _mm512_setzero_ps();

        let a_ptr = a.as_ptr() as *const u16;
        let b_ptr = b.as_ptr() as *const u16;

        for i in 0..chunks {
            let base = i * 64;

            let a0 = _mm512_cvtph_ps(_mm256_loadu_si256(a_ptr.add(base) as *const __m256i));
            let b0 = _mm512_cvtph_ps(_mm256_loadu_si256(b_ptr.add(base) as *const __m256i));
            acc0 = _mm512_fmadd_ps(a0, b0, acc0);

            let a1 = _mm512_cvtph_ps(_mm256_loadu_si256(a_ptr.add(base + 16) as *const __m256i));
            let b1 = _mm512_cvtph_ps(_mm256_loadu_si256(b_ptr.add(base + 16) as *const __m256i));
            acc1 = _mm512_fmadd_ps(a1, b1, acc1);

            let a2 = _mm512_cvtph_ps(_mm256_loadu_si256(a_ptr.add(base + 32) as *const __m256i));
            let b2 = _mm512_cvtph_ps(_mm256_loadu_si256(b_ptr.add(base + 32) as *const __m256i));
            acc2 = _mm512_fmadd_ps(a2, b2, acc2);

            let a3 = _mm512_cvtph_ps(_mm256_loadu_si256(a_ptr.add(base + 48) as *const __m256i));
            let b3 = _mm512_cvtph_ps(_mm256_loadu_si256(b_ptr.add(base + 48) as *const __m256i));
            acc3 = _mm512_fmadd_ps(a3, b3, acc3);
        }

        acc0 = _mm512_add_ps(acc0, acc1);
        acc2 = _mm512_add_ps(acc2, acc3);
        acc0 = _mm512_add_ps(acc0, acc2);

        let lane0 = _mm512_extractf32x4_ps(acc0, 0);
        let lane1 = _mm512_extractf32x4_ps(acc0, 1);
        let lane2 = _mm512_extractf32x4_ps(acc0, 2);
        let lane3 = _mm512_extractf32x4_ps(acc0, 3);
        let sum01 = _mm_add_ps(lane0, lane1);
        let sum23 = _mm_add_ps(lane2, lane3);
        let sum128 = _mm_add_ps(sum01, sum23);
        let shuf = _mm_movehdup_ps(sum128);
        let sums = _mm_add_ps(sum128, shuf);
        let shuf2 = _mm_movehl_ps(sums, sums);
        let result = _mm_add_ss(sums, shuf2);

        let mut dot = _mm_cvtss_f32(result) as f64;
        let tail_start = chunks * 64;
        for i in tail_start..len {
            let af = f16::from_bits(*a_ptr.add(i)).to_f32();
            let bf = f16::from_bits(*b_ptr.add(i)).to_f32();
            dot += (af * bf) as f64;
        }
        dot
    }
}
